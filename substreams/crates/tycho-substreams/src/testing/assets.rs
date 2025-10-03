pub fn read_block<B: prost::Message + Default>(filename: &str) -> B {
    use base64::Engine;

    let encoded = std::fs::read_to_string(filename).expect("Failed to read file");
    let raw_bytes = base64::prelude::BASE64_STANDARD
        .decode(&encoded)
        .expect("Failed to decode base64");

    B::decode(&*raw_bytes).expect("Not able to decode Block")
}
