use crate::{
    bluez::{enable_le_scan, get_filter, open, set_filter, HciEvent, HciFilter, HciType},
    bt_parsing::bt_parser,
    event::{Color, Dispatcher, Event},
    ibeacon_parsing::{ibeacon_parser, IBeacon},
};
use anyhow::{Context, Result};
use std::{
    convert::{TryFrom, TryInto},
    io::Read,
    os::unix::{io::FromRawFd, net::UnixStream},
    time::Duration,
};
use thiserror::Error;
use tracing::{debug, error};
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

pub fn run(dispatcher: &Dispatcher) -> Result<()> {
    let fd = open()?;

    let mut stream = unsafe { UnixStream::from_raw_fd(fd) };

    enable_le_scan(&mut stream)?;
    let old_filter = get_filter(&stream)?;
    set_filter(
        &stream,
        HciFilter::new(HciType::EventPkt, HciEvent::LeMetaEvent),
    )?;
    main_loop(&mut stream, &dispatcher).map_err(|e| {
        let _ = set_filter(&stream, old_filter);
        unsafe { libc::close(fd) };
        e
    })?;
    Ok(())
}

fn read_package(stream: &mut UnixStream, buf: &mut [u8]) -> Result<usize, anyhow::Error> {
    stream
        .read_exact(&mut buf[..3])
        .context("Couldn't read header from bluetooth socket")?;
    let len = 3 + buf[2] as usize;
    stream
        .read_exact(&mut buf[3..len])
        .context("Couldn't read body from bluetooth socket")?;
    Ok(len)
}

fn main_loop(stream: &mut UnixStream, dispatcher: &Dispatcher) -> Result<(), anyhow::Error> {
    let mut buf = [0u8; 258];
    loop {
        let len = read_package(stream, &mut buf)?;
        if let Ok((_, events)) = bt_parser()(&buf[..len]) {
            for event in events {
                debug!("Event {:?}", event);
                if let Ok((_, ibeacon)) = ibeacon_parser()(&event.data) {
                    if let Ok(event) = ibeacon.try_into() {
                        dispatcher.dispatch(&event);
                    }
                }
            }
        }
    }
}
