use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::io;

pub async fn start_server() -> io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:8080").await?;
    let mut buf = [0; 1024];
    let mut players: HashMap<SocketAddr, String> = HashMap::new();

    println!("Server running on 0.0.0.0:8080");
    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        let msg = String::from_utf8_lossy(&buf[..len]);

        if !players.contains_key(&addr) {
            let username = msg.trim().to_string();
            players.insert(addr, username.clone());
            println!("New player '{}' connected from {:?}", username, addr);
            sock.send_to(b"Welcome to the game!", addr).await?;
        } else {
            println!("Message from {}: {}", players[&addr], msg);
        }

        println!("Connected players:");
        for (addr, username) in &players {
            println!("{} at {:?}", username, addr);
        }
    }
}
