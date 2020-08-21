pub fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    return std::fs::read(filename).unwrap();
}