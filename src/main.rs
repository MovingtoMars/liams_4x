mod common;
mod client;
mod server;

use std::thread;

pub fn main() {
    let thread = thread::spawn(|| server::run_server());

    client::run_client();

    thread.join().unwrap();
}
