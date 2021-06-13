use nom::{
    combinator::{map, verify},
    number::complete::{be_u128, be_u16, be_u8},
    sequence::tuple,
    IResult,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct IBeacon {
    pub length: u8,
    pub r#type: u8,
    pub manufacturer_id: u16,
    pub sub_type: u8,
    pub sub_type_length: u8,
    pub proximity_uuid: Uuid,
    pub major: u16,
    pub minor: u16,
    pub signal_power: u8,
}

pub fn ibeacon_parser<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], IBeacon> {
    map(
        tuple((
            verify(be_u8, |l| *l == 0x1a_u8),
            verify(be_u8, |t| *t == 0xff_u8),
            verify(be_u16, |m| *m == 0x4c00_u16),
            verify(be_u8, |st| *st == 0x02_u8),
            verify(be_u8, |sl| *sl == 0x15_u8),
            map(be_u128, Uuid::from_u128),
            be_u16,
            be_u16,
            be_u8,
        )),
        |(
            length,
            r#type,
            manufacturer_id,
            sub_type,
            sub_type_length,
            proximity_uuid,
            major,
            minor,
            signal_power,
        )| IBeacon {
            length,
            r#type,
            manufacturer_id,
            sub_type,
            sub_type_length,
            proximity_uuid,
            major,
            minor,
            signal_power,
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_valid() -> Result<(), Box<dyn std::error::Error>> {
        ibeacon_parser()(
            b"\x1a\xffL\0\x02\x15\xa4\x95\xbb\x10\xc5\xb1KD\xb5\x12\x13p\xf0-t\xde\0:\x04,\xbf",
        )?;
        Ok(())
    }
}
