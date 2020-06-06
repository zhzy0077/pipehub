use crate::logger::ApplicationLogger;
use crate::models::WechatWork;
use crate::user::TENANT_ID_KEY;
use crate::{data, DbPool};
use actix_session::Session;
use actix_web::body::Body;
use actix_web::{get, put, web, Error as AWError, HttpRequest, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;

#[get("/wechat")]
pub async fn wechat(
    session: Session,
    pool: web::Data<DbPool>,
    logger: web::Data<Arc<ApplicationLogger>>,
    req: HttpRequest,
) -> std::result::Result<HttpResponse, AWError> {
    let request_id: Uuid = req
        .extensions()
        .get::<Uuid>()
        .cloned()
        .expect("No request id found.");
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        if let Some(wechat) =
            data::find_wechat_by_id(request_id, Arc::clone(&logger), pool, tenant_id).await?
        {
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
    logger: web::Data<Arc<ApplicationLogger>>,
    req: HttpRequest,
) -> std::result::Result<HttpResponse, AWError> {
    let request_id: Uuid = req
        .extensions()
        .get::<Uuid>()
        .cloned()
        .expect("No request id found.");
    if let Some(tenant_id) = session.get::<i64>(TENANT_ID_KEY)? {
        entity.tenant_id = tenant_id;
        entity.corp_id = entity.corp_id.trim().to_string();
        entity.secret = entity.secret.trim().to_string();
        data::upsert_wechat(request_id, Arc::clone(&logger), pool, entity).await?;
        Ok(HttpResponse::NoContent().body(Body::Empty))
    } else {
        Ok(HttpResponse::Unauthorized().body(Body::Empty))
    }
}
