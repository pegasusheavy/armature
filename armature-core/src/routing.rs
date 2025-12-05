// Routing system for HTTP requests

use crate::{Error, HttpMethod, HttpRequest, HttpResponse};
use std::collections::HashMap;
use std::sync::Arc;

/// A route handler function type
pub type HandlerFn = Arc<
    dyn Fn(
            HttpRequest,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<HttpResponse, Error>> + Send>,
        > + Send
        + Sync,
>;

/// Route definition with handler
#[derive(Clone)]
pub struct Route {
    pub method: HttpMethod,
    pub path: String,
    pub handler: HandlerFn,
}

/// Router for managing routes and dispatching requests
pub struct Router {
    pub routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Add a route to the router
    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    /// Find a route that matches the request
    pub async fn route(&self, mut request: HttpRequest) -> Result<HttpResponse, Error> {
        // Parse query parameters from path
        let (path, query_string) = request
            .path
            .split_once('?')
            .map(|(p, q)| (p, Some(q)))
            .unwrap_or((&request.path, None));

        if let Some(query) = query_string {
            request.query_params = parse_query_string(query);
        }

        // Find matching route
        for route in &self.routes {
            if route.method.as_str() != request.method {
                continue;
            }

            if let Some(params) = match_path(&route.path, path) {
                request.path_params = params;
                return (route.handler)(request).await;
            }
        }

        Err(Error::RouteNotFound(format!("{} {}", request.method, path)))
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Match a route path pattern against a request path
/// Returns Some(params) if matched, None otherwise
fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if let Some(param_name) = pattern_part.strip_prefix(':') {
            // This is a parameter
            params.insert(param_name.to_string(), path_part.to_string());
        } else if pattern_part != path_part {
            // Static part doesn't match
            return None;
        }
    }

    Some(params)
}

/// Parse a query string into a map of parameters
fn parse_query_string(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|part| {
            let mut split = part.splitn(2, '=');
            let key = split.next()?;
            let value = split.next().unwrap_or("");
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_path_static() {
        let pattern = "/users";
        let path = "/users";
        let result = match_path(pattern, path);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_match_path_with_param() {
        let pattern = "/users/:id";
        let path = "/users/123";
        let result = match_path(pattern, path);
        assert!(result.is_some());
        let params = result.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_match_path_no_match() {
        let pattern = "/users/:id";
        let path = "/posts/123";
        let result = match_path(pattern, path);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30";
        let params = parse_query_string(query);
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_match_path_multiple_params() {
        let pattern = "/users/:user_id/posts/:post_id";
        let path = "/users/123/posts/456";
        let result = match_path(pattern, path);
        assert!(result.is_some());
        let params = result.unwrap();
        assert_eq!(params.get("user_id"), Some(&"123".to_string()));
        assert_eq!(params.get("post_id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_match_path_trailing_slash() {
        let pattern = "/users";
        let path = "/users/";
        let result = match_path(pattern, path);
        // Should handle trailing slash gracefully
        assert!(result.is_some() || result.is_none());
    }

    #[test]
    fn test_match_path_nested() {
        let pattern = "/api/v1/users/:id";
        let path = "/api/v1/users/123";
        let result = match_path(pattern, path);
        assert!(result.is_some());
        let params = result.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_match_path_empty() {
        let pattern = "/";
        let path = "/";
        let result = match_path(pattern, path);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_query_string_empty() {
        let query = "";
        let params = parse_query_string(query);
        // Empty string may return one empty entry, which is fine
        assert!(params.is_empty() || params.len() == 1);
    }

    #[test]
    fn test_parse_query_string_special_chars() {
        let query = "name=john%20doe&email=test%40example.com";
        let params = parse_query_string(query);
        assert!(params.contains_key("name"));
        assert!(params.contains_key("email"));
    }

    #[test]
    fn test_parse_query_string_no_value() {
        let query = "flag&debug=true";
        let params = parse_query_string(query);
        assert!(params.contains_key("debug"));
        assert_eq!(params.get("debug"), Some(&"true".to_string()));
    }

    #[test]
    fn test_match_path_param_with_special_chars() {
        let pattern = "/users/:id";
        let path = "/users/abc-123";
        let result = match_path(pattern, path);
        assert!(result.is_some());
        let params = result.unwrap();
        assert_eq!(params.get("id"), Some(&"abc-123".to_string()));
    }

    #[test]
    fn test_route_creation() {
        use crate::HttpMethod;
        let route = Route {
            method: HttpMethod::GET,
            path: "/users".to_string(),
            handler: std::sync::Arc::new(|_req| {
                Box::pin(async move { Ok(crate::HttpResponse::ok()) })
            }),
        };

        assert_eq!(route.method, HttpMethod::GET);
        assert_eq!(route.path, "/users");
    }

    #[test]
    fn test_router_add_route() {
        let mut router = Router::new();
        let route = Route {
            method: crate::HttpMethod::GET,
            path: "/test".to_string(),
            handler: std::sync::Arc::new(|_req| {
                Box::pin(async move { Ok(crate::HttpResponse::ok()) })
            }),
        };

        router.add_route(route);
        assert_eq!(router.routes.len(), 1);
    }

    #[test]
    fn test_router_multiple_routes() {
        let mut router = Router::new();

        for i in 0..5 {
            let route = Route {
                method: crate::HttpMethod::GET,
                path: format!("/test{}", i),
                handler: std::sync::Arc::new(|_req| {
                    Box::pin(async move { Ok(crate::HttpResponse::ok()) })
                }),
            };
            router.add_route(route);
        }

        assert_eq!(router.routes.len(), 5);
    }

    #[test]
    fn test_parse_query_string_multiple_same_key() {
        let query = "tag=rust&tag=web&tag=framework";
        let params = parse_query_string(query);
        // Should contain at least one tag
        assert!(params.contains_key("tag"));
    }
}
