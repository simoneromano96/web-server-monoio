use crate::response_builder::ResponseBuilder;

use http_types::Response;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("Received an unserializable body {0}")]
    UnserializableBody(#[from] simd_json::Error),
}

/// Trait to implement on any struct to return a Response
pub trait IntoResponse {
    fn as_response(&self) -> Result<Response, SerializeError>;
}

impl<T> IntoResponse for T
where
    T: serde::Serialize,
{
    fn as_response(&self) -> Result<Response, SerializeError> {
        let body = simd_json::to_vec(&self)?;
        Ok(ResponseBuilder::json(body).build())
    }
}
