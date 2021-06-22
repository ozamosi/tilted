use super::{Emitter, EmitterConfig};
use crate::event::Event;
use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrometheusError {}

#[derive(Debug)]
pub struct Prometheus {
    address: String,
    temp_gauge_name: String,
    gravity_gauge_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PrometheusOptions {
    address: String,
    temp_gauge_name: String,
    gravity_gauge_name: String,
}

impl EmitterConfig for PrometheusOptions {
    fn get_emitter(&self) -> Result<Box<dyn Emitter>> {
        let p = Prometheus {
            address: self.address.to_string(),
            temp_gauge_name: self.temp_gauge_name.clone(),
            gravity_gauge_name: self.gravity_gauge_name.clone(),
        };
        Ok(Box::new(p))
    }
}

impl Emitter for Prometheus {
    fn emit(&self, event: &Event) -> Result<()> {
        let color: &'static str = (&event.color).into();
        let address = format!("{}/metrics/jobs/{}", self.address, "tilted");
        ureq::post(&address).send_string(&format!(
            "{}{{color={}}} {}",
            self.temp_gauge_name, color, event.temperature
        ))?;
        ureq::post(&address).send_string(&format!(
            "{}{{color={}}} {}",
            self.gravity_gauge_name, color, event.gravity
        ))?;
        Ok(())
    }
}
