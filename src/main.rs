
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate mio;
#[macro_use(quick_error)]
extern crate quick_error;

mod errors;
mod server;
mod traits;

use server::Server;

const IPV4_PORT : u32 = 67;

fn main() {
    let address = "0.0.0.0:6567".parse().unwrap();

    println!("yo ! {:?}", Server::new(address))
}

#[cfg(test)]
mod tests {
    pub use super::*;
}
