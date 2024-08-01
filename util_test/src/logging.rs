#[allow(unused_must_use)]
pub fn init_logging() {
    pretty_env_logger::try_init();
    println!();
}
