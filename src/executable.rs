use std::io::{Read, Write};

use crate::bit_utils::BitInformation;

#[derive(PartialEq, Debug)]
pub struct CythanCode {
    pub code: Vec<usize>,
    pub base: u8,
    pub start_pos: usize
}

#[test]
fn test() {
    let cyco = CythanCode {
        code: (0..1024).map(|x| x * 4).collect(),
        base: 4,
        start_pos: 35
    };
    let cyco1 = decode(&encode(&cyco)).unwrap();
    assert_eq!(cyco, cyco1);
    println!("{}",encode(&cyco).len());
}

pub fn encode(cc: &CythanCode) -> Vec<u8> {
    let mut vec = vec![0xC1,0x4B,0xA4,0x01];
    vec.push(cc.base);
    UnsignedVarInt(cc.start_pos as u32).encode(&mut vec).unwrap();
    for i in &cc.code {
        UnsignedVarInt(*i as u32).encode(&mut vec).unwrap();
    }
    vec
}
pub fn decode(data: &[u8]) -> Option<CythanCode> {
    let mut dec = data.iter();
    if vec![0xC1,0x4B,0xA4,0x01] != (0..4).flat_map(|_|Iterator::next(&mut dec)).copied().collect::<Vec<u8>>() {
        return None;
    }
    let base = *Iterator::next(&mut dec)?;
    let UnsignedVarInt(start_pos) = UnsignedVarInt::decode(&mut dec).ok()??;

    let mut code = Vec::new();
    while let Some(e) = UnsignedVarInt::decode(&mut dec).ok()? {
        code.push(e.0 as usize);
    }

    Some(CythanCode {
        code,
        base,
        start_pos: start_pos as usize,
    })

}

#[derive(Debug)]
pub struct UnsignedVarInt(pub u32);

impl UnsignedVarInt {
    fn decode(reader: &mut impl Reader) -> Result<Option<Self>, std::io::Error> {
        let mut shift_amount: u32 = 0;
        let mut decoded_value: u32 = 0;
        loop {
            let next_byte = if let Some(e) = reader.next() {
                e
            } else {
                return Ok(None);
            };
            decoded_value |= ((next_byte & 0b01111111) as u32) << shift_amount;
            if next_byte.has_most_signifigant_bit() {
                shift_amount += 7;
            } else {
                return Ok(Some(Self(decoded_value)));
            }
        }
    }

    fn encode(&self, writer: &mut impl Writer) -> Result<(), std::io::Error> {
        let mut value: u32 = self.0;
        if value == 0 {
            writer.write(0)?;
        } else {
            while value >= 0b10000000 {
                writer.write(((value & 0b01111111) as u8) | 0b10000000)?;
                value = value >> 7;
            }
            writer.write((value & 0b01111111) as u8)?;
        }
        Ok(())
    }
}

pub trait Reader {
    fn next(&mut self) -> Option<u8>;
}

pub trait Writer {
    fn write(&mut self, data: u8) -> Result<(), std::io::Error>;
    fn write_slice(&mut self, slice: &[u8]) -> Result<(), std::io::Error>;
}

impl<'a, T: Iterator<Item = &'a u8>> Reader for T {
    fn next(&mut self) -> Option<u8> {
        self.next()
            .copied()
    }
}

impl Writer for Vec<u8> {
    fn write(&mut self, data: u8) -> Result<(), std::io::Error> {
        self.push(data);
        Ok(())
    }

    fn write_slice(&mut self, slice: &[u8]) -> Result<(), std::io::Error> {
        self.extend_from_slice(slice);
        Ok(())
    }
}