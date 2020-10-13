use super::{Emitter, EmitterConfig};
use crate::event::Event;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum LogError {}

#[derive(Debug)]
pub struct Log {}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct LogOptions {} // Log level? Log format?

impl EmitterConfig for LogOptions {
    fn get_emitter(&self) -> Result<Box<dyn Emitter>> {
        Ok(Box::new(Log {}))
    }
}

#[async_trait]
impl Emitter for Log {
    async fn emit(&self, event: &Event) -> Result<()> {
        info!("Received event {:?}", event);
        Ok(())
    }
}
