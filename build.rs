fn main() {
    std::fs::create_dir_all(concat!("etc/", std::env!("CARGO_PKG_NAME")))
        .expect("Config dir to be created");
}
