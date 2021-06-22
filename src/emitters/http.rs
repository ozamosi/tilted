use super::{Emitter, EmitterConfig};
use crate::event::Event;
use anyhow::Result;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use thiserror::Error;
use tinytemplate::TinyTemplate;

#[derive(Error, Debug)]
pub enum HttpError {}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum Formats {
    Json,
    Query,
    Form,
}

#[derive(Debug)]
pub struct Http {
    method: String,
    uri: String,
    content_type: String,
    payload: HashMap<String, String>,
    last_emit: Arc<Mutex<SystemTime>>,
    min_interval: Duration,
    format: Formats,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct HttpOptions {
    #[serde(default = "default_method")]
    method: String,
    url: String,
    #[serde(rename = "content-type")]
    #[serde(default = "default_content_type")]
    content_type: String,
    #[serde(default = "default_format")]
    format: Formats,
    #[serde(with = "humantime_serde")]
    #[serde(default = "default_repeat")]
    #[serde(rename = "min-interval")]
    min_interval: Duration,
    payload: HashMap<String, String>,
}

fn default_content_type() -> String {
    "application/json".to_string()
}
fn default_format() -> Formats {
    Formats::Json
}
fn default_repeat() -> Duration {
    Duration::from_secs(5 * 60)
}
fn default_method() -> String {
    "POST".to_string()
}

impl EmitterConfig for HttpOptions {
    fn get_emitter(&self) -> Result<Box<dyn Emitter>> {
        Ok(Box::new(Http {
            method: self.method.clone(),
            uri: self.url.clone(),
            content_type: self.content_type.clone(),
            format: self.format.clone(),
            payload: self.payload.clone(),
            min_interval: self.min_interval,
            last_emit: Arc::new(Mutex::new(SystemTime::UNIX_EPOCH)),
        }))
    }
}

impl Emitter for Http {
    fn emit(&self, event: &Event) -> Result<()> {
        {
            let mut last_emit = self.last_emit.lock().unwrap();
            if *last_emit + self.min_interval > SystemTime::now() {
                return Ok(());
            }
            *last_emit = SystemTime::now();
        }
        let mut request =
            ureq::request(&self.method, &self.uri).set("Content-type", &self.content_type);
        let payload = self
            .payload
            .iter()
            .map(|(key, value)| {
                let mut tt = TinyTemplate::new();
                tt.add_template("key", key)?;
                tt.add_template("value", value)?;
                Ok((tt.render("key", event)?, tt.render("value", event)?))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        let request = match self.format {
            Formats::Json => request.send_json(ureq::serde_to_value(payload)?),
            Formats::Form => request.send_form(
                payload
                    .iter()
                    .map(|(x, y)| (x.as_ref(), y.as_ref()))
                    .collect::<Vec<_>>()
                    .as_ref(),
            ),
            Formats::Query => {
                for (key, val) in payload {
                    request = request.query(&key, &val);
                }
                request.send_string("")
            }
        };
        request?;
        Ok(())
    }
}
