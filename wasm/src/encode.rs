trait WasmEncode {
    fn encode(&self, bytes: &mut Vec<u8>);
}