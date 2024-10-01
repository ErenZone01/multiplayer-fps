use tokio::net::UdpSocket;
use std::io::{self, Write};
use std::net::SocketAddr;

pub async fn start_client() -> io::Result<()> { 
    print!("Enter IP Address (e.g., 127.0.0.1:8080): ");
    io::stdout().flush().unwrap();
    let mut ip_address = String::new();
    io::stdin().read_line(&mut ip_address).unwrap();
    let ip_address = ip_address.trim();
    let server_addr: SocketAddr = ip_address.parse().expect("Invalid IP address");

    print!("Enter your username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();

    let sock = UdpSocket::bind("0.0.0.0:0").await?;
    sock.send_to(username.as_bytes(), &server_addr).await?;

    let mut buf = [0; 1024];
    let (len, _) = sock.recv_from(&mut buf).await?;
    println!("Server response: {}", String::from_utf8_lossy(&buf[..len]));

    Ok(())
}
