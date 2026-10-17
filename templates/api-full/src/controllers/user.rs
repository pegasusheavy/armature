//! User controller

use crate::middleware::AuthMiddleware;
use crate::models::{
    ApiResponse, TokenClaims, UpdateUserRequest, UserResponse, UserRole, ValidationError,
};
use crate::services::get_user_service;
use armature::armature_validation::Validate;
use armature::prelude::*;
use armature::{HandlerFn, MiddlewareChain};
use std::future::Future;
use std::sync::Arc;
use uuid::Uuid;

#[controller("/api/users")]
#[derive(Default, Clone)]
pub struct UserController;

#[routes]
impl UserController {
    /// GET /api/users - list users (admin only)
    #[get("")]
    async fn list(req: HttpRequest) -> Result<HttpResponse, Error> {
        with_auth(req, |req| async move {
            let claims = verified_claims(&req)?;
            if claims.role != UserRole::Admin {
                return Err(Error::forbidden("Admin role required"));
            }

            let users: Vec<UserResponse> = get_user_service()
                .find_all()
                .iter()
                .map(UserResponse::from)
                .collect();

            HttpResponse::ok().with_json(&ApiResponse::success(users))
        })
        .await
    }

    /// GET /api/users/:id - get a user by id (self or admin)
    #[get("/:id")]
    async fn get(req: HttpRequest) -> Result<HttpResponse, Error> {
        with_auth(req, |req| async move {
            let id = parse_user_id(&req)?;
            authorize_self_or_admin(&req, id)?;

            match get_user_service().find_by_id(id) {
                Some(user) => {
                    HttpResponse::ok().with_json(&ApiResponse::success(UserResponse::from(&user)))
                }
                None => HttpResponse::not_found()
                    .with_json(&ApiResponse::<()>::error("NOT_FOUND", "User not found")),
            }
        })
        .await
    }

    /// PATCH /api/users/:id - update a user's profile (self or admin)
    #[patch("/:id")]
    async fn update(req: HttpRequest) -> Result<HttpResponse, Error> {
        with_auth(req, |req| async move {
            let id = parse_user_id(&req)?;
            authorize_self_or_admin(&req, id)?;

            let body: UpdateUserRequest = req.json().map_err(|e| {
                tracing::debug!(error = %e, "invalid update request body");
                Error::bad_request("Invalid request body")
            })?;
            if let Err(errors) = body.validate() {
                let details: Vec<ValidationError> = errors.into_iter().map(Into::into).collect();
                return HttpResponse::bad_request()
                    .with_json(&ApiResponse::<()>::validation_error(details));
            }

            match get_user_service().update(id, body.name) {
                Some(user) => {
                    HttpResponse::ok().with_json(&ApiResponse::success(UserResponse::from(&user)))
                }
                None => HttpResponse::not_found()
                    .with_json(&ApiResponse::<()>::error("NOT_FOUND", "User not found")),
            }
        })
        .await
    }

    /// DELETE /api/users/:id - delete a user (self or admin)
    #[delete("/:id")]
    async fn delete(req: HttpRequest) -> Result<HttpResponse, Error> {
        with_auth(req, |req| async move {
            let id = parse_user_id(&req)?;
            authorize_self_or_admin(&req, id)?;

            if get_user_service().delete(id) {
                Ok(HttpResponse::no_content())
            } else {
                HttpResponse::not_found()
                    .with_json(&ApiResponse::<()>::error("NOT_FOUND", "User not found"))
            }
        })
        .await
    }
}

/// The claims [`AuthMiddleware`] verified and attached to this request.
///
/// Absent only if a handler was reached without going through
/// [`with_auth`]; that is treated as unauthenticated (fail closed).
fn verified_claims(req: &HttpRequest) -> Result<&TokenClaims, Error> {
    req.extension::<TokenClaims>()
        .ok_or_else(|| Error::unauthorized("Authentication required"))
}

/// Allow the request only if the caller is the user `target` or an admin.
///
/// The authorization decision uses the verified token's `sub`/`role`, never
/// anything else the client sent.
fn authorize_self_or_admin(req: &HttpRequest, target: Uuid) -> Result<(), Error> {
    let claims = verified_claims(req)?;
    if claims.role == UserRole::Admin {
        return Ok(());
    }
    match Uuid::parse_str(&claims.sub) {
        Ok(caller) if caller == target => Ok(()),
        _ => Err(Error::forbidden("You may only access your own account")),
    }
}

fn parse_user_id(req: &HttpRequest) -> Result<Uuid, Error> {
    let id_str = req
        .param("id")
        .ok_or_else(|| Error::bad_request("User ID required"))?;

    Uuid::parse_str(id_str).map_err(|_| Error::bad_request("Invalid user ID format"))
}

/// Runs `handler` behind [`AuthMiddleware`], so every route that goes
/// through this requires a valid bearer token.
async fn with_auth<F, Fut>(req: HttpRequest, handler: F) -> Result<HttpResponse, Error>
where
    F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<HttpResponse, Error>> + Send + 'static,
{
    let mut chain = MiddlewareChain::new();
    chain.use_middleware(AuthMiddleware);

    let boxed: HandlerFn = Arc::new(move |req: HttpRequest| {
        Box::pin(handler(req))
            as std::pin::Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
    });
    chain.apply(req, boxed).await
}
