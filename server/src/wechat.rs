use crate::data::Pool;
use crate::models::WechatWork;
use crate::user::TENANT_ID_KEY;
use crate::RequestId;
use actix_session::Session;
use actix_web::{get, put, web, Error as AWError, HttpResponse};
use log::info;

#[get("/wechat")]
pub async fn wechat(
    session: Session,
    pool: web::Data<Pool>,
    request_id: RequestId,
) -> std::result::Result<HttpResponse, AWError> {
    info!("{} [Get WeChat]", request_id);
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(wechat) = pool.find_wechat_by_id(tenant_id).await? {
            info!("{} [Get WeChat] {}", request_id, wechat.tenant_id);
            Ok(HttpResponse::Ok().json(wechat))
        } else {
            Ok(HttpResponse::Ok().json(WechatWork::default()))
        }
    } else {
        Ok(HttpResponse::Unauthorized().body(()))
    }
}

#[put("/wechat")]
pub async fn update(
    session: Session,
    pool: web::Data<Pool>,
    web::Json(mut entity): web::Json<WechatWork>,
    request_id: RequestId,
) -> std::result::Result<HttpResponse, AWError> {
    info!("{} [Update WeChat]", request_id);
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        info!("{} [Update WeChat] {}", request_id, tenant_id);
        entity.tenant_id = tenant_id;
        entity.corp_id = entity.corp_id.trim().to_string();
        entity.secret = entity.secret.trim().to_string();
        pool.upsert_wechat(entity).await?;
        Ok(HttpResponse::NoContent().body(()))
    } else {
        Ok(HttpResponse::Unauthorized().body(()))
    }
}
