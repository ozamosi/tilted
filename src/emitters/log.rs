use super::{Emitter, EmitterConfig};
use crate::event::Event;
use anyhow::Result;
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

impl Emitter for Log {
    fn emit(&self, event: &Event) -> Result<()> {
        info!("Received event {:?}", event);
        Ok(())
    }
}
