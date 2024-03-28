use std::env;

#[cfg(test)]
pub fn enable_test_log() {
    env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init()
}