use crate::db::models::Server;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

/// Perform a basic TCP health check on the server's SSH port.
/// Falls back to TCP connect check since russh requires key setup.
pub async fn check_health(server: &Server) -> bool {
    let addr = format!("{}:{}", server.host, server.port);

    match addr.parse::<SocketAddr>() {
        Ok(socket_addr) => {
            match timeout(Duration::from_secs(5), TcpStream::connect(socket_addr)).await {
                Ok(Ok(_)) => true,
                _ => false,
            }
        }
        Err(_) => {
            // Try DNS resolution for hostnames
            match timeout(Duration::from_secs(5), TcpStream::connect(addr)).await {
                Ok(Ok(_)) => true,
                _ => false,
            }
        }
    }
}
