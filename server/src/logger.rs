use crate::config::{LogConfig, Logger};
use crate::error::Result;
use actix_web::web;
use appinsights::telemetry::SeverityLevel;
use appinsights::{InMemoryChannel, TelemetryClient};
use log::{Level, Log, Metadata, Record};
use simplelog::{Config, TermLogger, TerminalMode};

pub async fn configure_logger(log_config: &LogConfig) -> Result<()> {
    let log_config = log_config.clone();
    web::block(move || -> Result<()> {
        match log_config.logger {
            Logger::ApplicationInsight => {
                ApplicationInsightLogger::init(log_config.instrumentation_key, log_config.level)
                    .expect("Fail to init AppInsight logger.")
            }
            Logger::TermLogger => TermLogger::init(
                log_config.level.to_level_filter(),
                Config::default(),
                TerminalMode::Mixed,
            )
            .expect("Fail to init Term logger."),
        };
        Ok(())
    })
    .await?;

    Ok(())
}

pub struct ApplicationInsightLogger {
    client: TelemetryClient<InMemoryChannel>,
    level: Level,
}

impl ApplicationInsightLogger {
    pub fn init(i_key: String, level: Level) -> Result<()> {
        log::set_max_level(level.to_level_filter());
        log::set_boxed_logger(Box::new(ApplicationInsightLogger {
            client: TelemetryClient::new(i_key),
            level,
        }))
        .expect("Fail to bind logger.");
        Ok(())
    }
}

impl Log for ApplicationInsightLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{}", record.args());
            let severity = record.severity();
            self.client.track_trace(msg, severity);
        }
    }

    fn flush(&self) {
        self.client.flush_channel();
    }
}

trait SeverityAware {
    fn severity(&self) -> SeverityLevel;
}

impl<'a> SeverityAware for Record<'a> {
    fn severity(&self) -> SeverityLevel {
        match self.level() {
            Level::Error => SeverityLevel::Error,
            Level::Warn => SeverityLevel::Warning,
            Level::Info => SeverityLevel::Information,
            Level::Debug => SeverityLevel::Verbose,
            Level::Trace => SeverityLevel::Verbose,
        }
    }
}
