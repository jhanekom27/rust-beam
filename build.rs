use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    let password = env::var("SECRET_PASSWORD")
        .expect("SECRET_PASSWORD environment variable not set");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("password.rs");
    let mut password_file = File::create(&dest_path).unwrap();
    write!(password_file, "pub const PASSWORD: &str = {:?};", password)
        .unwrap();
}
