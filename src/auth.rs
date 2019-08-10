use actix_web::{
    error::Result,
    middleware::{Middleware, Started},
    HttpRequest, HttpResponse,
};

use crate::api::ErrorResponse;

#[derive(Clone, Debug)]
pub(crate) struct ApiKeyAuth(pub(crate) String);

impl<S: 'static> Middleware<S> for ApiKeyAuth {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        if let Some(key) = req.headers().get("X-Auth-Token") {
            if key == &self.0 {
                return Ok(Started::Done);
            }
        }

        let error = ErrorResponse {
            error: "Invalid credentials".to_owned(),
            error_code: 401,
        };

        Ok(Started::Response(HttpResponse::Unauthorized().json(error)))
    }
}
