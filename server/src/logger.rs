use crate::config::LogConfig;
use crate::error::Result;
use actix_http::http::Uri;
use actix_web::web;
use appinsights::telemetry::{
    RemoteDependencyTelemetry, RequestTelemetry, SeverityLevel, Telemetry, TraceTelemetry,
};
use appinsights::{InMemoryChannel, TelemetryClient};
use log::{Level, Log, RecordBuilder};
use simplelog::{Config, TermLogger, TerminalMode};
use std::time::Duration;
use uuid::Uuid;

pub struct ApplicationLogger {
    app_insight: Option<TelemetryClient<InMemoryChannel>>,
    term_logger: Option<TermLogger>,
}

impl ApplicationLogger {
    pub fn track_trace(&self, id: Uuid, level: Level, message: &str) {
        if let Some(ref app_insight) = self.app_insight {
            let mut event = TraceTelemetry::new(message, ApplicationLogger::severity(&level));
            event
                .properties_mut()
                .insert("request_id".to_string(), id.to_string());
            app_insight.track(event);
        }

        if let Some(ref term_logger) = self.term_logger {
            term_logger.log(
                &RecordBuilder::new()
                    .level(level)
                    .args(format_args!("{} {}", id, message))
                    .build(),
            );
        }
    }

    pub fn track_request(
        &self,
        id: Uuid,
        method: &str,
        uri: Uri,
        duration: Duration,
        response_code: &str,
    ) {
        let name = format!("{} {}", method, uri);
        if let Some(ref app_insight) = self.app_insight {
            let event =
                RequestTelemetry::new_request(id, name, uri.clone(), duration, response_code);
            app_insight.track(event);
        }

        if let Some(ref term_logger) = self.term_logger {
            term_logger.log(
                &RecordBuilder::new()
                    .level(Level::Info)
                    .args(format_args!(
                        "{} {} {} {}",
                        method,
                        uri,
                        duration.as_millis(),
                        response_code
                    ))
                    .build(),
            );
        }
    }

    pub fn track_dependency(
        &self,
        id: Uuid,
        name: &str,
        dependency_type: &str,
        duration: Duration,
        target: &str,
        result_code: &str,
        data: &str,
        success: bool,
    ) {
        if let Some(ref app_insight) = self.app_insight {
            let mut event = RemoteDependencyTelemetry::new_dependency(
                name,
                dependency_type,
                duration,
                target,
                result_code,
                data,
                success,
            );
            event
                .properties_mut()
                .insert("request_id".to_string(), id.to_string());
            app_insight.track(event);
        }

        if let Some(ref term_logger) = self.term_logger {
            term_logger.log(
                &RecordBuilder::new()
                    .level(Level::Info)
                    .args(format_args!(
                        "{} {} {} {} {} {}",
                        dependency_type,
                        target,
                        name,
                        duration.as_millis(),
                        result_code,
                        data
                    ))
                    .build(),
            );
        }
    }

    fn severity(level: &Level) -> SeverityLevel {
        match level {
            Level::Error => SeverityLevel::Error,
            Level::Warn => SeverityLevel::Warning,
            Level::Info => SeverityLevel::Information,
            Level::Debug => SeverityLevel::Verbose,
            Level::Trace => SeverityLevel::Verbose,
        }
    }
}

impl ApplicationLogger {
    pub async fn new(log_config: &LogConfig) -> Self {
        let i_key = log_config.instrumentation_key.clone();
        let level = log_config.level.clone();
        web::block(move || -> Result<ApplicationLogger> {
            let app_insight = if i_key != String::default() {
                Some(TelemetryClient::new(i_key.clone()))
            } else {
                None
            };
            let term_logger = Some(*TermLogger::new(
                level.to_level_filter(),
                Config::default(),
                TerminalMode::Mixed,
            ));
            TermLogger::init(
                level.to_level_filter(),
                Config::default(),
                TerminalMode::Mixed,
            )
            .expect("Unable to bind term logger.");

            Ok(ApplicationLogger {
                app_insight,
                term_logger,
            })
        })
        .await
        .expect("Failed to initialize logger.")
    }
}
