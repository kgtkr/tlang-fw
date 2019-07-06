use byteorder::{LittleEndian, WriteBytesExt};


trait BinaryEncode {
    fn encode(&self, bytes: &mut Vec<u8>);
}

fn encodeUint8(x: u8, bytes: &mut Vec<u8>) {
    bytes.write_u8(x).unwrap();
}

fn encodeUint16(x: u16, bytes: &mut Vec<u8>) {
    bytes.write_u16::<LittleEndian>(x).unwrap();
}

fn encodeUint32(x: u32, bytes: &mut Vec<u8>) {
    bytes.write_u32::<LittleEndian>(x).unwrap();
}