pub mod http;
pub mod log;
pub mod prometheus;

use super::event::Event;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;

trait EmitterConfig {
    fn get_emitter(&self) -> Result<Box<dyn Emitter>>;
}

pub trait Emitter: Debug {
    fn emit(&self, event: &Event) -> Result<()>;
}

pub fn init(config: &HashMap<String, Emitters>) -> Result<Vec<Box<dyn Emitter>>> {
    let mut result: Vec<Box<dyn Emitter>> = vec![];

    for module in config.values() {
        match module {
            Emitters::Http(module) => module.get_emitter().map(|m| result.push(m))?,
            Emitters::Log(module) => module.get_emitter().map(|m| result.push(m))?,
            Emitters::Prometheus(module) => module.get_emitter().map(|m| result.push(m))?,
        }
    }
    Ok(result)
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(tag = "emitter")]
pub enum Emitters {
    #[serde(rename = "http")]
    Http(crate::emitters::http::HttpOptions),
    #[serde(rename = "log")]
    Log(crate::emitters::log::LogOptions),
    #[serde(rename = "prometheus")]
    Prometheus(crate::emitters::prometheus::PrometheusOptions),
}
