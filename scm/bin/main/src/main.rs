use std::env;
use std::process;

mod api;
mod commands;
mod core;

use api::{Client, DispatchRequest};
use core::client::BrowseClient;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match BrowseClient.dispatch(DispatchRequest { args }) {
        Ok(response) => process::exit(response.exit_code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(e.exit_code());
        }
    }
}
