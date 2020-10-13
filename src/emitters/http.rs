use super::{Emitter, EmitterConfig};
use crate::event::Event;
use anyhow::Result;
use async_trait::async_trait;
use handlebars::Handlebars;
use http::method::{InvalidMethod, Method};
use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::sync::Mutex;
use url::{ParseError, Url};

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Please provide the method (e.g. POST/GET) to use for communicating with the service")]
    MethodError(#[from] InvalidMethod),
    #[error("Please provide the URL to connect to")]
    UrlError(#[from] ParseError),
    #[error(transparent)]
    ReqwestException(#[from] ReqwestError),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum Formats {
    Json,
    Query,
    Form,
}

#[derive(Debug)]
pub struct Http {
    client: Client,
    method: Method,
    uri: Url,
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
            client: Client::new(),
            method: Method::from_str(&self.method).map_err(HttpError::from)?,
            uri: Url::parse(&self.url).map_err(HttpError::from)?,
            content_type: self.content_type.clone(),
            format: self.format.clone(),
            payload: self.payload.clone(),
            min_interval: self.min_interval,
            last_emit: Arc::new(Mutex::new(SystemTime::UNIX_EPOCH)),
        }))
    }
}

#[async_trait]
impl Emitter for Http {
    async fn emit(&self, event: &Event) -> Result<()> {
        let mut last_emit = self.last_emit.lock().await;
        if *last_emit + self.min_interval > SystemTime::now() {
            return Ok(());
        }
        *last_emit = SystemTime::now();
        let request = self
            .client
            .request(self.method.clone(), self.uri.clone())
            .header("Content-type", &self.content_type);
        let reg = Handlebars::new();
        let payload = self
            .payload
            .iter()
            .map(|(key, value)| {
                Ok((
                    reg.render_template(key, event)?,
                    reg.render_template(value, event)?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        let request = match self.format {
            Formats::Json => request.json(&payload),
            Formats::Query => request.query(&payload),
            Formats::Form => request.form(&payload),
        };
        request.send().await.map_err(HttpError::from)?;
        Ok(())
    }
}
