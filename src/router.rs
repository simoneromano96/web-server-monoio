use crate::handler::{self, Handler};
use crate::parse::ParsedRequest;
use crate::response_builder::ResponseBuilder;

use http_types::{Method, Response, StatusCode, Version};
use std::collections::HashMap;
use thiserror::Error;

type PathHandler = HashMap<String, Box<dyn Handler>>;

#[derive(Debug, Error)]
enum RouterError {
    #[error("Handler not found")]
    HandlerNotFound,
}

async fn default_not_found_handler(_request: ParsedRequest) -> handler::HandlerResult {
    let mut res = Response::new(StatusCode::NotFound);
    res.set_version(Some(Version::Http1_1));
    res.set_body("Page not found");
    Ok(res)
}

async fn default_error_handler(_request: ParsedRequest) -> handler::HandlerResult {
    Ok(ResponseBuilder::internal_server_error().build())
}

pub struct Router {
    routes: HashMap<Method, PathHandler>,
    not_found_handler: Box<dyn Handler>,
    error_handler: Box<dyn Handler>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            not_found_handler: Box::new(default_not_found_handler),
            error_handler: Box::new(default_error_handler),
        }
    }

    pub fn add<T>(
        &mut self,
        method: &Method,
        path: &str,
        // Pin<Box<dyn Future<Output=()> + 'a>>
        handler: T,
    ) where
        T: Handler,
    {
        match self.routes.get_mut(method) {
            Some(path_map) => {
                path_map.insert(path.to_string(), Box::new(handler));
            }
            None => {
                let mut path_map = PathHandler::new();
                path_map.insert(path.to_string(), Box::new(handler));
                self.routes.insert(*method, path_map);
            }
        }
    }

    pub async fn handle_route(&self, parsed_request: ParsedRequest) -> handler::HandlerResult {
        let ParsedRequest { method, path, .. } = &parsed_request;
        let handler = self
            .resolve_handler(method, path)
            .unwrap_or(&self.not_found_handler);

        handler.call(parsed_request).await
    }

    fn resolve_handler(
        &self,
        method: &Method,
        path: &str,
    ) -> Result<&Box<dyn Handler>, RouterError> {
        let (_, handler) = self
            .routes
            .iter()
            .find(|(routes_method, _routes)| *routes_method == method)
            .ok_or(RouterError::HandlerNotFound)?
            .1
            .iter()
            .find(|(route_path, _handler)| *route_path == path)
            .ok_or(RouterError::HandlerNotFound)?;

        Ok(handler)
    }
}
