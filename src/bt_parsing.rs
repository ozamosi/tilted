use nom::{
    bytes::complete::take,
    combinator::{all_consuming, flat_map, map, map_parser, verify},
    multi::count,
    number::complete::{be_i8, be_u8},
    sequence::{preceded, tuple},
    IResult,
};
use num_traits::FromPrimitive;
use std::convert::TryInto;

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq)]
enum PacketType {
    Command = 0x01,
    AsynchronousData = 0x02,
    SynchronousData = 0x03,
    Event = 0x04,
    ExtendedCommand = 0x09,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq)]
enum EventCode {
    LeMeta = 0x3e,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq)]
enum LeEventSubcode {
    ConnectionCompleteEvent = 0x01,
    AdvertisingReport = 0x02,
}

#[derive(Debug)]
pub struct LeEvent {
    event_type: EventType,
    address_type: AddressType,
    address: [u8; 6],
    pub data: Vec<u8>,
    rssi: i8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
enum EventType {
    AdvInd = 0x00,
    AdvDirectInd = 0x01,
    AdvScanInd = 0x02,
    AdvNonConnInd = 0x03,
    ScanRsp = 0x04,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
enum AddressType {
    PublicDevice = 0x00,
    RandomDevice = 0x01,
    PublicIdentity = 0x02,
    RandomIdentity = 0x03,
}

fn n_le_reports<'a>(num_reports: usize) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<LeEvent>> {
    map(
        tuple((
            count(map(be_u8, FromPrimitive::from_u8), num_reports),
            count(map(be_u8, FromPrimitive::from_u8), num_reports),
            count(take(6_usize), num_reports),
            flat_map(count(be_u8, num_reports), take_nums),
            count(be_i8, num_reports),
        )),
        |(event_types, address_types, addresses, datum, rssis)| {
            event_types
                .iter()
                .zip(address_types)
                .zip(addresses)
                .zip(datum)
                .zip(rssis)
                // XXX this throws away misunderstood data, which is surprising
                .flat_map(|((((event_type, address_type), address), data), rssi)| {
                    Some(LeEvent {
                        event_type: *event_type.as_ref()?,
                        address_type: address_type?,
                        address: address.try_into().ok()?,
                        data: data.to_vec(),
                        rssi,
                    })
                })
                .collect::<Vec<_>>()
        },
    )
}

fn take_nums<'a>(nums: Vec<u8>) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<&'a [u8]>> {
    move |i| {
        let mut i = i;
        let out = nums
            .iter()
            .map(|n| {
                let (new_i, data) = take(*n as usize)(i)?;
                i = new_i;
                Ok(data)
            })
            .collect::<Result<Vec<&[u8]>, _>>()?;
        Ok((i, out))
    }
}

fn advertising_report_parser<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<LeEvent>> {
    flat_map(be_u8, |x| n_le_reports(x.into()))
}

fn le_event_parser<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<LeEvent>> {
    preceded(
        verify(be_u8, |e| {
            FromPrimitive::from_u8(*e) == Some(LeEventSubcode::AdvertisingReport)
        }),
        advertising_report_parser(),
    )
}

fn event_parser<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<LeEvent>> {
    preceded(
        verify(be_u8, |e| {
            FromPrimitive::from_u8(*e) == Some(EventCode::LeMeta)
        }),
        map_parser(flat_map(be_u8, take), all_consuming(le_event_parser())),
    )
}

pub fn bt_parser<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<LeEvent>> {
    all_consuming(preceded(
        verify(be_u8, |e| {
            FromPrimitive::from_u8(*e) == Some(PacketType::Event)
        }),
        event_parser(),
    ))
}
