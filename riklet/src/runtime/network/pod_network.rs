use async_trait::async_trait;
use std::fmt::Debug;

use super::{Result, RuntimeNetwork};

#[derive(Debug)]
pub struct PodRuntimeNetwork {}

impl PodRuntimeNetwork {
    pub fn new() -> Self {
        PodRuntimeNetwork {}
    }
}

#[async_trait]
impl RuntimeNetwork for PodRuntimeNetwork {
    async fn init(&self) -> Result<()> {
        todo!()
    }

    async fn destroy(&self) -> Result<()> {
        todo!()
    }
}
