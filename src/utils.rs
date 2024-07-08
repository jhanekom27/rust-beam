pub fn get_random_name() -> String {
    memorable_wordlist::kebab_case(30)
}

pub fn get_key_from_buf(buf: &[u8]) -> String {
    String::from_utf8(buf.to_vec())
        .expect("Invalid UTF-8 Sequence")
        .trim_end_matches("\0")
        .to_string()
}
