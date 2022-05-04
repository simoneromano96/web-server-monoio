use http_types::{mime, Body, Response, StatusCode, Version};

pub struct ResponseBuilder {
    response: Response,
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        let mut response = Response::new(StatusCode::Ok);
        response.set_version(Some(Version::Http1_1));
        Self { response }
    }
}

impl ResponseBuilder {
    fn set_status(mut self, status: StatusCode) -> Self {
        self.response.set_status(status);
        self
    }

    pub fn internal_server_error() -> Self {
        Self::default().set_status(StatusCode::InternalServerError)
    }

    pub fn json<T: Into<Body>>(body: T) -> Self {
        let mut response = Self::default();
        response.response.set_body(body);
        response.response.set_content_type(mime::JSON);
        response
    }

    pub fn build(self) -> Response {
        self.response
    }
}
