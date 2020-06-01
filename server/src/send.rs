use crate::error::{Error, Result};
use crate::models::WechatWork;
use crate::{data, DbPool};
use actix_http::client::Connector;
use actix_web::body::Body;
use actix_web::client::Client;
use actix_web::{web, Error as AWError, HttpResponse};
use base58::FromBase58;
use serde::Deserialize;
use serde::Serialize;
use std::convert::TryInto;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct WeChatAccessToken {
    #[serde(rename = "errcode")]
    error_code: u64,
    #[serde(rename = "errmsg")]
    error_message: String,
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Serialize)]
struct WeChatMessage {
    #[serde(rename = "touser")]
    to_user: String,
    #[serde(rename = "agentid")]
    agent_id: i64,
    #[serde(rename = "msgtype")]
    message_type: String,
    text: WeChatMessageText,
    #[serde(serialize_with = "crate::util::bool_to_int")]
    enable_duplicate_check: bool,
    duplicate_check_interval: u64,
}

#[derive(Debug, Serialize)]
struct WeChatMessageText {
    content: String,
}

#[derive(Debug, Deserialize)]
struct WeChatSendResponse {
    #[serde(rename = "errcode")]
    error_code: u64,
    #[serde(rename = "errmsg")]
    error_message: String,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    text: Option<String>,
}

pub async fn send(
    pool: web::Data<DbPool>,
    key: web::Path<String>,
    payload: web::Bytes,
    web::Query(message): web::Query<Message>,
) -> std::result::Result<HttpResponse, AWError> {
    let app_key = key.into_inner().from_base58().map_err(|e| Error::from(e))?;
    let app_id = i64::from_le_bytes((&app_key[0..8]).try_into().expect("Unexpected"));

    if let Some(wechat) = data::find_wechat_by_app_id(pool, app_id).await? {
        let token = get_token(&wechat).await?;
        if let Message { text: Some(text) } = message {
            do_send(&wechat, &token, text).await?;
        } else if let Ok(text) = String::from_utf8(payload.to_vec()) {
            do_send(&wechat, &token, text).await?;
        }
    }

    Ok(HttpResponse::Ok().body(Body::Empty))
}

async fn get_token(wechat: &WechatWork) -> Result<WeChatAccessToken> {
    let corpid = &wechat.corp_id;
    let secret = &wechat.secret;

    let client = Client::default();
    let url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={}&corpsecret={}",
        corpid, secret
    );

    let mut response = client.get(url).send().await?;
    let token: WeChatAccessToken = response.json().await?;

    Ok(token)
}

async fn do_send(wechat: &WechatWork, token: &WeChatAccessToken, msg: String) -> Result<()> {
    let connector = Connector::new().timeout(Duration::from_secs(10)).finish();
    let client = Client::build()
        .timeout(Duration::from_secs(10))
        .connector(connector)
        .finish();
    let url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
        token.access_token
    );
    let mut response = client
        .post(url)
        .send_json(&WeChatMessage {
            to_user: "@all".to_string(),
            agent_id: wechat.agent_id,
            message_type: "text".to_string(),
            text: WeChatMessageText { content: msg },
            enable_duplicate_check: false,
            duplicate_check_interval: 0,
        })
        .await?;

    let _: WeChatSendResponse = response.json().await?;

    Ok(())
}
