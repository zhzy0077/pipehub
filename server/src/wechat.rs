use crate::data::Pool;
use crate::models::WechatWork;
use crate::user::TENANT_ID_KEY;
use actix_session::Session;
use actix_web::body::Body;
use actix_web::{get, put, web, Error as AWError, HttpResponse};

#[get("/wechat")]
pub async fn wechat(session: Session, pool: Pool) -> std::result::Result<HttpResponse, AWError> {
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(wechat) = pool.find_wechat_by_id(tenant_id).await? {
            Ok(HttpResponse::Ok().json(wechat))
        } else {
            Ok(HttpResponse::Ok().json(WechatWork::default()))
        }
    } else {
        Ok(HttpResponse::Unauthorized().body(Body::Empty))
    }
}

#[put("/wechat")]
pub async fn update(
    session: Session,
    pool: Pool,
    web::Json(mut entity): web::Json<WechatWork>,
) -> std::result::Result<HttpResponse, AWError> {
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        entity.tenant_id = tenant_id;
        entity.corp_id = entity.corp_id.trim().to_string();
        entity.secret = entity.secret.trim().to_string();
        pool.upsert_wechat(entity).await?;
        Ok(HttpResponse::NoContent().body(Body::Empty))
    } else {
        Ok(HttpResponse::Unauthorized().body(Body::Empty))
    }
}
