use crate::bt_parsing::bt_parser;
use crate::event::{Color, Dispatcher, Event};
use crate::ibeacon_parsing::{ibeacon_parser, IBeacon};
use anyhow::Result;
use nom::error::ErrorKind;
use std::convert::{TryFrom, TryInto};
use std::ffi::c_void;
use std::os::unix::{
    io::{FromRawFd, RawFd},
    net::UnixStream as StdUnixStream,
};
use thiserror::Error;
use tokio::{
    io::{self, AsyncReadExt, BufReader},
    net::UnixStream,
};
use tracing::error;
use uuid::Uuid;

use libbluetooth::{
    bluetooth::{bdaddr_t, SOL_HCI},
    hci::{
        hci_filter, EVT_LE_META_EVENT, HCI_EVENT_PKT, HCI_FILTER, OCF_LE_SET_SCAN_ENABLE,
        OGF_LE_CTL,
    },
    hci_lib::{
        hci_filter_set_event, hci_filter_set_ptype, hci_get_route, hci_open_dev, hci_send_cmd,
    },
};

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
    let mut x = bdaddr_t { b: [0; 6] };
    let idx = unsafe { hci_get_route(&mut x) };
    let fd: RawFd = unsafe { hci_open_dev(idx) };

    if fd < 0 {
        return Err(io::Error::last_os_error().into());
    }

    unsafe {
        hci_send_cmd(
            fd,
            OGF_LE_CTL as u16,
            OCF_LE_SET_SCAN_ENABLE as u16,
            1,
            &mut 1u32 as *mut u32 as *mut c_void,
        )
    };

    let mut filter: hci_filter = Default::default();
    hci_filter_set_event(EVT_LE_META_EVENT, &mut filter);
    hci_filter_set_ptype(HCI_EVENT_PKT, &mut filter);
    unsafe {
        libc::setsockopt(
            fd,
            SOL_HCI,
            HCI_FILTER,
            &filter as *const hci_filter as *const c_void,
            std::mem::size_of::<hci_filter>() as u32,
        )
    };

    let stream = UnixStream::from_std(unsafe { StdUnixStream::from_raw_fd(fd) })?;
    let mut reader = BufReader::new(stream);

    main_loop(&mut reader, &dispatcher).await;
    Ok(())
}

async fn main_loop<'a>(reader: &mut BufReader<UnixStream>, dispatcher: &Dispatcher) {
    let mut buf = [0u8; 255];
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
