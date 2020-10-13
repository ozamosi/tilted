use crate::bluez::{enable_le_scan, open, set_filter};
use crate::bt_parsing::bt_parser;
use crate::event::{Color, Dispatcher, Event};
use crate::ibeacon_parsing::{ibeacon_parser, IBeacon};
use anyhow::Result;
use nom::error::ErrorKind;
use std::convert::{TryFrom, TryInto};
use std::os::unix::{io::FromRawFd, net::UnixStream as StdUnixStream};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::UnixStream,
};
use tracing::error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Unknown UUID {0} - don't understand what color this is")]
    UnknownUuidError(Uuid),
}

// List from https://kvurd.com/blog/tilt-hydrometer-ibeacon-data-format/
pub const RED_UUID: Uuid = Uuid::from_bytes([
    164, 149, 187, 16, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116, 222,
]);
pub const GREEN_UUID: Uuid = Uuid::from_bytes([
    164, 149, 187, 32, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116, 222,
]);

pub const BLACK_UUID: Uuid = Uuid::from_bytes([
    164, 149, 187, 48, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116, 222,
]);

pub const PURPLE_UUID: Uuid = Uuid::from_bytes([
    164, 149, 187, 64, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116, 222,
]);

pub const ORANGE_UUID: Uuid = Uuid::from_bytes([
    164, 149, 187, 80, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116, 222,
]);

pub const BLUE_UUID: Uuid = Uuid::from_bytes([
    164, 149, 187, 96, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116, 222,
]);

pub const YELLOW_UUID: Uuid = Uuid::from_bytes([
    164, 149, 187, 112, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116, 222,
]);

pub const PINK_UUID: Uuid = Uuid::from_bytes([
    164, 149, 187, 128, 197, 177, 75, 68, 181, 18, 19, 112, 240, 45, 116, 222,
]);

impl TryFrom<Uuid> for Color {
    type Error = EventError;

    fn try_from(uuid: Uuid) -> Result<Color, EventError> {
        match uuid {
            RED_UUID => Ok(Color::Red),
            GREEN_UUID => Ok(Color::Green),
            BLACK_UUID => Ok(Color::Black),
            PURPLE_UUID => Ok(Color::Purple),
            ORANGE_UUID => Ok(Color::Orange),
            BLUE_UUID => Ok(Color::Blue),
            YELLOW_UUID => Ok(Color::Yellow),
            PINK_UUID => Ok(Color::Pink),
            e => Err(EventError::UnknownUuidError(e)),
        }
    }
}

impl TryFrom<IBeacon> for Event {
    type Error = EventError;

    fn try_from(ibeacon: IBeacon) -> Result<Event, EventError> {
        Ok(Event {
            color: ibeacon.proximity_uuid.try_into()?,
            temperature: ibeacon.major,
            gravity: (ibeacon.minor as f32) / 1000.,
        })
    }
}

pub async fn run(dispatcher: &Dispatcher) -> Result<()> {
    let fd = open()?;

    let mut stream = UnixStream::from_std(unsafe { StdUnixStream::from_raw_fd(fd) })?;
    set_filter(&stream)?;
    enable_le_scan(&mut stream).await?;
    let mut reader = BufReader::new(stream);

    main_loop(&mut reader, &dispatcher).await;
    Ok(())
}

async fn main_loop<'a>(reader: &mut BufReader<UnixStream>, dispatcher: &Dispatcher) {
    let mut buf = [0u8; 258];
    loop {
        let response = reader.read_exact(&mut buf[..3]).await;
        if let Err(e) = response {
            error!("Couldn't read header from bluetooth socket: {}", e);
            return;
        }
        let len = 3 + buf[2] as usize;
        let response = reader.read_exact(&mut buf[3..len]).await;
        if let Err(e) = response {
            error!("Couldn't read body from bluetooth socket: {}", e);
            return;
        }
        if let Ok((_, events)) = bt_parser::<(&[u8], ErrorKind)>()(&buf[..len]) {
            for event in events {
                if let Ok((_, ibeacon)) = ibeacon_parser::<(&[u8], ErrorKind)>()(&event.data) {
                    if let Ok(event) = ibeacon.try_into() {
                        dispatcher.dispatch(&event).await;
                    }
                }
            }
        }
    }
}
