use crate::error::Error;
use crate::error::Result;
use actix_http::http::header;
use reqwest::Client;
use serde::Deserialize;
use std::str::FromStr;
use url::Url;

lazy_static! {
    static ref AUTH_URL: Url = Url::parse("https://github.com/login/oauth/authorize").unwrap();
}

pub struct GitHubClient {
    client_id: String,
    client_secret: String,
    callback_url: Url,
}

impl GitHubClient {
    pub fn new(client_id: String, client_secret: String, callback_url: impl Into<String>) -> Self {
        GitHubClient {
            client_id,
            client_secret,
            callback_url: Url::from_str(&callback_url.into()).expect("Malformed callback url."),
        }
    }

    pub fn authorize_url(&self, state: &String) -> Url {
        let mut url = AUTH_URL.clone();
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("state", state)
            .append_pair("redirect_uri", self.callback_url.as_str());

        url
    }

    pub async fn exchange_code(&self, http_client: &Client, code: &str) -> Result<String> {
        let response = http_client
            .post("https://github.com/login/oauth/access_token")
            .query(&[
                ("client_id", &self.client_id[..]),
                ("client_secret", &self.client_secret[..]),
                ("code", code),
            ])
            .header(header::ACCEPT, "application/json")
            .send()
            .await?;
        let access_token: GitHubAccessToken = response.json().await?;
        Ok(access_token.access_token)
    }

    pub async fn get_user(&self, http_client: &Client, token: &str) -> Result<GithubUser> {
        let response = http_client
            .get("https://api.github.com/user")
            .header(header::USER_AGENT, "PipeHub")
            .header(header::AUTHORIZATION, format!("token {}", token))
            .send()
            .await
            .map_err(Error::from)?;
        let github_user = response.json::<GithubUser>().await?;
        Ok(github_user)
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub id: i64,
}

#[derive(Debug, Deserialize)]
struct GitHubAccessToken {
    access_token: String,
}

// {"access_token":"e72e16c7e42f292c6912e7710c838347ae178b4a", "scope":"repo,gist", "token_type":"bearer"}
