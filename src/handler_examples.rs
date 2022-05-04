use crate::handler::HandlerResult;
use crate::into_response::IntoResponse;
use crate::parse::ParsedRequest;
use crate::response_builder;

use http_types;
use http_types::{Body, Response, StatusCode, Version};
use serde::{Deserialize, Serialize};

pub async fn test_handler(_request: ParsedRequest) -> HandlerResult {
    let mut res = Response::new(StatusCode::Ok);
    res.set_version(Some(Version::Http1_1));
    res.set_body("Hello, world!");
    res.append_header("Content-Type", "text/plain");
    Ok(res)
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct TestJsonBody {
    pub test: String,
    pub hello: String,
}

impl Into<Body> for TestJsonBody {
    fn into(self) -> Body {
        simd_json::to_vec(&self).unwrap().into()
    }
}

pub async fn another_handler(_request: ParsedRequest) -> HandlerResult {
    let mut res = Response::new(StatusCode::Ok);
    res.set_version(Some(Version::Http1_1));
    res.set_body(TestJsonBody {
        test: "Hello".to_string(),
        hello: "This is test json body".to_string(),
    });
    res.set_content_type(http_types::mime::JSON);

    Ok(res)
}

pub async fn body_handler(mut request: ParsedRequest) -> HandlerResult {
    let body: TestJsonBody = simd_json::from_slice(&mut request.body).unwrap();
    let mut res = Response::new(StatusCode::Ok);
    res.set_version(Some(Version::Http1_1));
    res.set_body(body);
    res.set_content_type(http_types::mime::JSON);

    Ok(res)
}

pub async fn response_builder(_request: ParsedRequest) -> HandlerResult {
    let body = TestJsonBody {
        test: "Hello".to_string(),
        hello: "This is test json body".to_string(),
    };
    Ok(response_builder::ResponseBuilder::json(body).build())
}

pub async fn response_builder_trait(_request: ParsedRequest) -> HandlerResult {
    let body = TestJsonBody {
        test: "Hello".to_string(),
        hello: "This is test json body".to_string(),
    };
    Ok(body.as_response()?)
}
