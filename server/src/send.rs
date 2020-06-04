use crate::error::{Error, Result};
use crate::logger::ApplicationLogger;
use crate::models::WechatWork;
use crate::{data, AccessTokenCache, DbPool, Response};
use actix_web::{web, Error as AWError, HttpRequest, HttpResponse};
use base58::FromBase58;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct WeChatAccessToken {
    #[serde(rename = "errcode")]
    error_code: u64,
    #[serde(rename = "errmsg")]
    error_message: String,
    access_token: String,
    #[serde(rename = "expires_in", deserialize_with = "crate::util::expires_at")]
    expires_at: Instant,
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
    logger: web::Data<Arc<ApplicationLogger>>,
    access_token_cache: web::Data<Arc<AccessTokenCache>>,
    http_client: web::Data<Client>,
    req: HttpRequest,
) -> std::result::Result<HttpResponse, AWError> {
    let request_id: Uuid = req
        .extensions()
        .get::<Uuid>()
        .cloned()
        .expect("No request id found.");
    let app_key = key.into_inner().from_base58().map_err(|e| Error::from(e))?;
    let app_id = i64::from_le_bytes((&app_key[0..8]).try_into().expect("Unexpected"));

    let wechat = data::find_wechat_by_app_id(request_id, Arc::clone(&logger), pool, app_id)
        .await?
        .ok_or_else(|| Error::User("Unknown APP ID."))?;
    let mut token = access_token_cache.get(&app_id);
    if token.is_none() || token.as_ref().unwrap().expires_at.le(&Instant::now()) {
        let new_token = get_token(&http_client, request_id, &logger, &wechat).await?;
        access_token_cache.insert(app_id, new_token);
        token = access_token_cache.get(&app_id);
    }
    let message = if let Message { text: Some(text) } = message {
        text
    } else if let Ok(text) = String::from_utf8(payload.to_vec()) {
        text
    } else {
        return Err(Error::User("No message is provided."))?;
    };

    let mut token = token.unwrap();
    let mut retry_count = 0;
    while let Err(e) = do_send(
        &http_client,
        request_id,
        &logger,
        &wechat,
        token.value(),
        message.clone(),
    )
    .await
    {
        if retry_count > 3 {
            return Err(e)?;
        } else {
            retry_count += 1;
        }
        let new_token = get_token(&http_client, request_id, &logger, &wechat).await?;
        access_token_cache.insert(app_id, new_token);
        token = access_token_cache.get(&app_id).unwrap();
    }

    Ok(HttpResponse::Ok().json(Response {
        request_id,
        success: true,
        error_message: "".to_owned(),
        hint: format!("Retried {} times.", retry_count),
    }))
}

async fn get_token(
    client: &Client,
    request_id: Uuid,
    logger: &ApplicationLogger,
    wechat: &WechatWork,
) -> Result<WeChatAccessToken> {
    let corpid = &wechat.corp_id;
    let secret = &wechat.secret;

    let start = Instant::now();
    let url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={}&corpsecret={}",
        corpid, secret
    );

    let response = client.get(&url).send().await?;
    let token: WeChatAccessToken = response.json().await?;

    logger.track_dependency(
        request_id,
        "GET https://qyapi.weixin.qq.com/cgi-bin/gettoken",
        "HTTPS",
        start.elapsed(),
        "qyapi.weixin.qq.com",
        &token.error_message,
        &url,
        token.error_code == 0,
    );

    Ok(token)
}

async fn do_send(
    client: &Client,
    request_id: Uuid,
    logger: &ApplicationLogger,
    wechat: &WechatWork,
    token: &WeChatAccessToken,
    msg: String,
) -> Result<()> {
    let start = Instant::now();
    let url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
        token.access_token
    );
    let response = client
        .post(&url)
        .json(&WeChatMessage {
            to_user: "@all".to_string(),
            agent_id: wechat.agent_id,
            message_type: "text".to_string(),
            text: WeChatMessageText { content: msg },
            enable_duplicate_check: false,
            duplicate_check_interval: 0,
        })
        .send()
        .await?;

    let reply: WeChatSendResponse = response.json().await?;

    logger.track_dependency(
        request_id,
        "POST https://qyapi.weixin.qq.com/cgi-bin/message/send",
        "HTTPS",
        start.elapsed(),
        "qyapi.weixin.qq.com",
        &reply.error_message,
        &url,
        reply.error_code == 0,
    );

    Ok(())
}
