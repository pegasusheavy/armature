//! User controller

use crate::models::{ApiResponse, UserResponse};
use crate::services::{AuthService, UserService};
use armature::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub struct UserController {
    user_service: Arc<UserService>,
    auth_service: Arc<AuthService>,
}

impl UserController {
    pub fn new(user_service: Arc<UserService>, auth_service: Arc<AuthService>) -> Self {
        Self {
            user_service,
            auth_service,
        }
    }

    fn verify_auth(&self, request: &HttpRequest) -> Result<(), HttpResponse> {
        let auth_header = request.headers.get("Authorization").or_else(|| {
            request.headers.get("authorization")
        });

        match auth_header {
            Some(token) => {
                self.auth_service.verify_token(token).map_err(|e| {
                    HttpResponse::unauthorized()
                        .json(ApiResponse::<()>::error("UNAUTHORIZED", e))
                })?;
                Ok(())
            }
            None => Err(HttpResponse::unauthorized()
                .json(ApiResponse::<()>::error("UNAUTHORIZED", "Missing authorization header"))),
        }
    }

    fn handle_list(&self, request: &HttpRequest) -> HttpResponse {
        if let Err(response) = self.verify_auth(request) {
            return response;
        }

        let users: Vec<UserResponse> = self
            .user_service
            .find_all()
            .iter()
            .map(UserResponse::from)
            .collect();

        HttpResponse::json(ApiResponse::success(users))
    }

    fn handle_get(&self, request: &HttpRequest) -> HttpResponse {
        if let Err(response) = self.verify_auth(request) {
            return response;
        }

        let id = match request.path_params.get("id") {
            Some(id_str) => match Uuid::parse_str(id_str) {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::bad_request()
                        .json(ApiResponse::<()>::error("BAD_REQUEST", "Invalid user ID format"));
                }
            },
            None => {
                return HttpResponse::bad_request()
                    .json(ApiResponse::<()>::error("BAD_REQUEST", "User ID required"));
            }
        };

        match self.user_service.find_by_id(id) {
            Some(user) => HttpResponse::json(ApiResponse::success(UserResponse::from(&user))),
            None => {
                HttpResponse::not_found().json(ApiResponse::<()>::error("NOT_FOUND", "User not found"))
            }
        }
    }

    fn handle_delete(&self, request: &HttpRequest) -> HttpResponse {
        if let Err(response) = self.verify_auth(request) {
            return response;
        }

        let id = match request.path_params.get("id") {
            Some(id_str) => match Uuid::parse_str(id_str) {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::bad_request()
                        .json(ApiResponse::<()>::error("BAD_REQUEST", "Invalid user ID format"));
                }
            },
            None => {
                return HttpResponse::bad_request()
                    .json(ApiResponse::<()>::error("BAD_REQUEST", "User ID required"));
            }
        };

        if self.user_service.delete(id) {
            HttpResponse::no_content()
        } else {
            HttpResponse::not_found().json(ApiResponse::<()>::error("NOT_FOUND", "User not found"))
        }
    }
}

impl Controller for UserController {
    fn routes(&self) -> Vec<Route> {
        vec![
            Route::new(HttpMethod::GET, "/api/users", "list"),
            Route::new(HttpMethod::GET, "/api/users/:id", "get"),
            Route::new(HttpMethod::DELETE, "/api/users/:id", "delete"),
        ]
    }

    fn handle(&self, route_name: &str, request: &HttpRequest) -> HttpResponse {
        match route_name {
            "list" => self.handle_list(request),
            "get" => self.handle_get(request),
            "delete" => self.handle_delete(request),
            _ => HttpResponse::not_found(),
        }
    }
}

