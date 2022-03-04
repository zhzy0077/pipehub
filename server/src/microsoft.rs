use crate::error::Result;
use actix_web::http::header;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use url::Url;

lazy_static! {
    static ref AUTH_URL: Url =
        Url::parse("https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize").unwrap();
}

pub struct MicrosoftClient {
    client_id: String,
    client_secret: String,
    callback_url: Url,
}

impl MicrosoftClient {
    pub fn new(client_id: String, client_secret: String, callback_url: impl Into<String>) -> Self {
        MicrosoftClient {
            client_id,
            client_secret,
            callback_url: Url::from_str(&callback_url.into()).expect("Malformed callback url."),
        }
    }

    pub fn authorize_url(&self, state: &str) -> Url {
        let mut url = AUTH_URL.clone();
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("state", state)
            .append_pair("redirect_uri", self.callback_url.as_str())
            .append_pair("response_type", "code")
            .append_pair("response_mode", "query")
            .append_pair("scope", "Tasks.ReadWrite,offline_access");

        url
    }

    pub async fn exchange_refresh_code(&self, http_client: &Client, code: &str) -> Result<String> {
        let response = http_client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&[
                ("client_id", &self.client_id[..]),
                ("client_secret", &self.client_secret[..]),
                ("code", code),
                ("scope", "Tasks.ReadWrite,offline_access"),
                ("grant_type", "authorization_code"),
                ("redirect_uri", self.callback_url.as_str()),
            ])
            .header(header::ACCEPT, "application/json")
            .send()
            .await?;
        let text = response.text().await?;
        println!("{:?}", text);
        let refresh_token: MicrosoftToken = serde_json::from_str(&text)?;
        Ok(refresh_token.refresh_token)
    }

    pub async fn post_task(
        &self,
        http_client: &Client,
        refresh_token: &str,
        list_id: &str,
        title: &str,
        content: &str,
    ) -> Result<()> {
        let response = http_client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&[
                ("client_id", &self.client_id[..]),
                ("client_secret", &self.client_secret[..]),
                ("refresh_token", refresh_token),
                ("scope", "Tasks.ReadWrite"),
                ("grant_type", "refresh_token"),
            ])
            .header(header::ACCEPT, "application/json")
            .send()
            .await?;
        let access_token: MicrosoftToken = response.json().await?;

        let response = http_client
            .post(&format!(
                "https://graph.microsoft.com/v1.0/me/todo/lists/{}/tasks",
                list_id
            ))
            .header(
                "Authorization",
                format!("Bearer {}", access_token.access_token),
            )
            .json(&MicrosoftTodoTask {
                title: title.to_string(),
                body: MicrosoftTodoTaskBody {
                    content: content.to_string(),
                },
            })
            .send()
            .await?;

        let _: MicrosoftTodoTask = response.json().await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MicrosoftTodoTask {
    pub title: String,
    pub body: MicrosoftTodoTaskBody,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MicrosoftTodoTaskBody {
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct MicrosoftToken {
    access_token: String,
    #[serde(default)]
    refresh_token: String,
}
