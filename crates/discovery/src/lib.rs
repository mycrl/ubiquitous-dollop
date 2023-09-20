use std::net::{Ipv4Addr, SocketAddr};

use tokio::net::UdpSocket;

pub struct Discovery {
    socket: UdpSocket,
}

impl Discovery {
    pub const PORTS: [u16; 3] = [17017, 17117, 17217];

    pub async fn broadcast(&self, bytes: &[u8]) {
        for port in Self::PORTS {
            self.socket
                .send_to(bytes, SocketAddr::from((Ipv4Addr::BROADCAST, port)))
                .await
                .unwrap();
        }
    }
}
