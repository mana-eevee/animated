use std::error::Error;
use tokio::net::TcpStream;

pub struct PeerConnection {
    pub stream: TcpStream,
}

impl PeerConnection {
    pub fn new(stream: TcpStream) -> Self {
        return PeerConnection { stream: stream };
    }
}

pub trait ConnectablePeer {
    fn ip(&self) -> String; 
    fn port(&self) -> u16;
}

pub async fn connect_peer(peer: &dyn ConnectablePeer) -> Result<PeerConnection, Box<dyn Error>> {
    let stream = TcpStream::connect(format!("{}:{}", peer.ip(), peer.port())).await?;
    return Ok(PeerConnection::new(stream));
}