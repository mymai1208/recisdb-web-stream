use std::sync::Mutex;

use mirakurun_conf::MirakurunConfing;
use once_cell::sync::Lazy;

mod mirakurun_conf;
mod recisdb;
mod server;

pub const MAX_BUFFER_SIZE: usize = 1024 * 1024 * 60;

pub static CONFIG: Lazy<Mutex<MirakurunConfing>> = Lazy::new(|| {
    Mutex::new(MirakurunConfing::new())
});

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    CONFIG.lock().unwrap().load_mirakurun_conf().expect("Failed to load mirakurun config");

    server::init_server().await;
}
