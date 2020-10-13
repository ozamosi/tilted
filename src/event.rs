use crate::emitters::Emitter;
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;
use tracing::warn;

#[derive(Debug, Serialize)]
pub struct Event {
    pub color: Color,
    pub temperature: u16, // Farenheight
    pub gravity: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Red,
    Green,
    Black,
    Purple,
    Orange,
    Blue,
    Yellow,
    Pink,
}

impl<'a> From<&'a Color> for &'static str {
    fn from(color: &'a Color) -> &'static str {
        match color {
            Color::Red => "red",
            Color::Green => "green",
            Color::Black => "black",
            Color::Purple => "purple",
            Color::Orange => "orange",
            Color::Blue => "blue",
            Color::Yellow => "yellow",
            Color::Pink => "pink",
        }
    }
}

pub struct Dispatcher {
    pub modules: Vec<Box<dyn Emitter>>,
}

impl Dispatcher {
    pub async fn dispatch(&self, event: &Event) {
        for module in &self.modules {
            match timeout(Duration::from_secs(10), module.emit(event)).await {
                Ok(_) => (),
                Err(e) => warn!("Couldn't send event: {}", e),
            }
        }
    }
}
