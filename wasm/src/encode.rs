trait BinaryEncode {
    fn encode(&self, bytes: &mut Vec<u8>);
}