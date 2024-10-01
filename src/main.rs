// src/main.rs
use std::env;
use std::io;

mod servers; 

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--client" {
        servers::client::start_client().await?;
    } else { 
        servers::server::start_server().await?; 
    }
    Ok(())
}
