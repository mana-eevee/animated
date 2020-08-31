use std::error::Error;
use tokio::net::TcpStream;

pub trait ConnectablePeer {
    fn ip(&self) -> String; 
    fn port(&self) -> u16;
}

pub async fn connect_peer(peer: &dyn ConnectablePeer) -> Result<TcpStream, Box<dyn Error>> {
    let stream = TcpStream::connect(format!("{}:{}", peer.ip(), peer.port())).await?;
    return Ok(stream);
}