use super::{Emitter, EmitterConfig};
use crate::event::Event;
use anyhow::Result;
use async_trait::async_trait;
use prometheus::{self, GaugeVec, Opts, Registry};
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrometheusError {
    #[error(transparent)]
    PrometheusError(#[from] prometheus::Error),
}

#[derive(Debug)]
pub struct Prometheus {
    address: String,
    registry: Registry,
    temp_gauge: GaugeVec,
    gravity_gauge: GaugeVec,
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
            temp_gauge: GaugeVec::new(
                Opts::new(self.temp_gauge_name.clone(), "The temperature reported")
                    .variable_labels(vec!["color".to_owned()]),
                &["color"],
            )
            .map_err(PrometheusError::from)?,
            gravity_gauge: GaugeVec::new(
                Opts::new(self.gravity_gauge_name.clone(), "The gravity reported")
                    .variable_labels(vec!["color".to_owned()]),
                &["color"],
            )
            .map_err(PrometheusError::from)?,
            registry: Registry::new(),
        };
        p.registry
            .register(Box::new(p.temp_gauge.clone()))
            .map_err(PrometheusError::from)?;
        p.registry
            .register(Box::new(p.gravity_gauge.clone()))
            .map_err(PrometheusError::from)?;
        Ok(Box::new(p))
    }
}

#[async_trait]
impl Emitter for Prometheus {
    async fn emit(&self, event: &Event) -> Result<()> {
        let color: &'static str = (&event.color).into();
        self.temp_gauge
            .with_label_values(&[color])
            .set(event.temperature.into());
        self.gravity_gauge
            .with_label_values(&[color])
            .set(event.gravity.into());
        let metric_families = self.registry.gather();
        prometheus::push_metrics_async(
            "tilted",
            HashMap::new(),
            &self.address,
            metric_families,
            None,
        )
        .await?;
        Ok(())
    }
}
