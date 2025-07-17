use std::pin::Pin;
use async_trait::async_trait;

use crate::{control_block::ControlBlock, engine::engine::Engine};

#[derive(Clone, Debug)]
pub struct ReturnCode {
    pub success: bool,
    pub payload: Option<String>,
    pub control_block: Option<ControlBlock>,
}

#[async_trait]
pub trait AsyncHandler: Send + Sync {
    async fn call(&self, input: String) -> ReturnCode;
}

pub struct HandlerWrapper<F>(pub F)
where 
    F: Fn(String) -> Pin<Box<dyn Future<Output = ReturnCode> + Send>> + Send + Sync + 'static;

#[async_trait]
impl<F> AsyncHandler for HandlerWrapper<F>
where 
    F: Fn(String) -> Pin<Box<dyn Future<Output = ReturnCode> + Send>> + Send + Sync + 'static,
{
    async fn call(&self, input: String) -> ReturnCode {
        (self.0)(input).await
    }
}

pub type Handler = Box<dyn AsyncHandler>;

impl Engine {
    
}

pub fn into_handler<F, Fut>(f: F) -> Handler
where
    F: Fn(String) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ReturnCode> + Send + 'static,
{
    Box::new(HandlerWrapper(move |input| {
        Box::pin(f(input))
    }))
}

