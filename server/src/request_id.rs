use crate::error::Error;
use actix_http::http::HeaderValue;
use actix_http::{HttpMessage, Payload};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error as AWError, FromRequest, HttpRequest};
use futures_util::future::{ok, ready, Ready};
use reqwest::header::HeaderName;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct RequestId(Uuid);

impl Display for RequestId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hyphenated())
    }
}

impl RequestId {
    pub fn new() -> Self {
        RequestId(Uuid::new_v4())
    }
}

impl FromRequest for RequestId {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            req.extensions()
                .get::<RequestId>()
                .map(RequestId::clone)
                .ok_or(Error::Unexpected("Missing request id.".to_string())),
        )
    }
}

pub struct RequestIdAware;

impl<S, B> Transform<S> for RequestIdAware
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = AWError>,
    S::Future: 'static,
    B: 'static,
{
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Transform = RequestIdMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // associate with the request
        ok(RequestIdMiddleware {
            service,
            id: RequestId::new(),
        })
    }
}

pub struct RequestIdMiddleware<S> {
    service: S,
    id: RequestId,
}

impl<S, B> Service for RequestIdMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = AWError>,
    S::Future: 'static,
    B: 'static,
{
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        req.extensions_mut().insert(self.id.clone());
        let fut = self.service.call(req);

        let request_id = self.id.to_string();

        Box::pin(async move {
            let mut res = fut.await?;

            res.headers_mut().insert(
                HeaderName::from_static("x-request-id"),
                HeaderValue::from_str(&request_id).unwrap(),
            );

            Ok(res)
        })
    }
}
