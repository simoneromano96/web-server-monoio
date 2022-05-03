use crate::parse::ParsedRequest;
use async_trait::async_trait;
use http_types::Response;
use std::future::Future;

pub type HandlerResult = Result<Response, ()>;

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    /// Invoke the endpoint within the given context
    async fn call(&self, req: ParsedRequest) -> HandlerResult;
}

#[async_trait]
impl<F, Fut> Handler for F
where
    F: Send + Sync + 'static + Fn(ParsedRequest) -> Fut,
    Fut: Future<Output = HandlerResult> + Send + 'static,
{
    async fn call(&self, req: ParsedRequest) -> HandlerResult {
        let fut = (self)(req);
        let res = fut.await?;
        Ok(res.into())
    }
}