use crate::data::Pool;
use crate::error::{Error, Result};
use crate::models::WechatWork;
use crate::{AccessTokenCache, Response};
use actix_web::{web, Error as AWError, HttpResponse};
use base58::FromBase58;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

use std::time::Instant;

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
    to_user: Option<String>,
    #[serde(rename = "toparty")]
    to_party: Option<String>,
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
    to_party: Option<String>,
}

pub async fn send(
    pool: web::Data<Pool>,
    key: web::Path<String>,
    payload: web::Bytes,
    web::Query(message): web::Query<Message>,
    access_token_cache: web::Data<AccessTokenCache>,
    http_client: web::Data<Client>,
) -> std::result::Result<HttpResponse, AWError> {
    let app_key = key.into_inner().from_base58().map_err(Error::from)?;
    let app_id = i64::from_le_bytes((&app_key[0..8]).try_into().expect("Unexpected"));

    let tenant = pool
        .find_tenant_by_app_id(app_id)
        .await?
        .ok_or(Error::User("Unknown APP ID."))?;
    let wechat = pool
        .find_wechat_by_app_id(app_id)
        .await?
        .ok_or(Error::User("No WeChat credentials configured."))?;

    let text = if let Message {
        text: Some(text),
        to_party: _,
    } = message
    {
        text
    } else if let Ok(text) = String::from_utf8(payload.to_vec()) {
        text
    } else {
        return Err(Error::User("No message is provided.").into());
    };

    if tenant
        .block_list
        .split(',')
        .map(|word| word.trim())
        .filter(|word| !word.is_empty())
        .any(|block_word| text.contains(block_word))
    {
        return Err(Error::User("Message blocked.").into());
    }

    let mut token = access_token_cache.get(&app_id);
    if token.is_none() || token.as_ref().unwrap().expires_at.le(&Instant::now()) {
        let new_token = get_token(&http_client, &wechat).await?;
        access_token_cache.insert(app_id, new_token);
        token = access_token_cache.get(&app_id);
    }

    let mut token = token.unwrap();
    let mut retry_count = 0;
    while let Err(e) = do_send(
        &http_client,
        &wechat,
        token.value(),
        text.clone(),
        message.to_party.clone(),
    )
    .await
    {
        if retry_count > 3 {
            return Err(e.into());
        } else {
            retry_count += 1;
        }
        let new_token = get_token(&http_client, &wechat).await?;
        access_token_cache.insert(app_id, new_token);
        token = access_token_cache.get(&app_id).unwrap();
    }

    Ok(HttpResponse::Ok().json(Response {
        success: true,
        error_message: "".to_owned(),
    }))
}

async fn get_token(client: &Client, wechat: &WechatWork) -> Result<WeChatAccessToken> {
    let corpid = &wechat.corp_id;
    let secret = &wechat.secret;

    let _start = Instant::now();
    let url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={}&corpsecret={}",
        corpid, secret
    );

    let response = client.get(&url).send().await?;
    let token: WeChatAccessToken = response.json().await?;

    if token.error_code != 0 {
        return Err(Error::Dependency(token.error_message));
    }

    Ok(token)
}

async fn do_send(
    client: &Client,
    wechat: &WechatWork,
    token: &WeChatAccessToken,
    msg: String,
    to_party: Option<String>,
) -> Result<WeChatSendResponse> {
    let url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
        token.access_token
    );
    let response = client
        .post(&url)
        .json(&WeChatMessage {
            to_user: match to_party {
                Some(_) => None,
                None => Some("@all".to_owned()),
            },
            to_party,
            agent_id: wechat.agent_id,
            message_type: "text".to_string(),
            text: WeChatMessageText { content: msg },
            enable_duplicate_check: false,
            duplicate_check_interval: 0,
        })
        .send()
        .await?;

    let response: WeChatSendResponse = response.json().await?;

    if response.error_code != 0 {
        return Err(Error::Dependency(response.error_message));
    }

    Ok(response)
}
