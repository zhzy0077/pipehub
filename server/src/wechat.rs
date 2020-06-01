use crate::models::WechatWork;
use crate::user::TENANT_ID_KEY;
use crate::{data, DbPool};
use actix_session::Session;
use actix_web::body::Body;
use actix_web::{get, put, web, Error as AWError, HttpResponse};

#[get("/wechat")]
pub async fn wechat(
    session: Session,
    pool: web::Data<DbPool>,
) -> std::result::Result<HttpResponse, AWError> {
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(wechat) = data::find_wechat_by_id(pool, tenant_id).await? {
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
    pool: web::Data<DbPool>,
    web::Json(mut entity): web::Json<WechatWork>,
) -> std::result::Result<HttpResponse, AWError> {
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        entity.tenant_id = tenant_id;
        data::upsert_wechat(pool, entity).await?;
        Ok(HttpResponse::NoContent().body(Body::Empty))
    } else {
        Ok(HttpResponse::Unauthorized().body(Body::Empty))
    }
}
