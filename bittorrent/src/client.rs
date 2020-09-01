use crate::p2p::{
    deserialize_peer_handshake, gen_peer_handshake, serialize_peer_handshake, PeerHandshake,
    HANDSHAKE_BYTE_SIZE,
};
use crate::torrent::TorrentMetainfo;

use std::error::Error;
use std::fmt;
use tokio::net::TcpStream;
use tokio::prelude::*;

pub trait ConnectablePeer {
    fn ip(&self) -> String;
    fn port(&self) -> u16;
}

impl fmt::Display for dyn ConnectablePeer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}:{}", self.ip(), self.port());
    }
}

pub async fn connect_peer(peer: &dyn ConnectablePeer) -> Result<TcpStream, Box<dyn Error>> {
    let stream = TcpStream::connect(format!("{}:{}", peer.ip(), peer.port())).await?;
    return Ok(stream);
}

fn truncate_or_pad(candidate: &str, width: usize) -> String {
    if candidate.len() > width {
        return String::from(&candidate[0..width]);
    } else {
        return format!("{:0>width$}", candidate, width = width);
    }
}

pub fn gen_peer_id() -> String {
    return truncate_or_pad("animate-test", 20);
}

pub fn gen_peer_id_bytes() -> [u8; 20] {
    let peer_id = gen_peer_id();
    let mut bytes: [u8; 20] = [0x0; 20];
    bytes.copy_from_slice(peer_id.as_bytes());
    return bytes;
}

pub async fn try_connect(peer: &dyn ConnectablePeer) -> Result<TcpStream, ()> {
    let maybe_connection = connect_peer(peer).await;

    match maybe_connection {
        Ok(conn) => {
            debug!("Connected to peer {}:{}!", peer.ip(), peer.port());
            return Ok(conn);
        }
        Err(e) => {
            error!(
                "Failed to connect to peer {}:{}! Error: {:?}",
                peer.ip(),
                peer.port(),
                e
            );
            return Err(());
        }
    }
}

pub async fn try_handshake(conn: &mut TcpStream, handshake: &PeerHandshake) -> Result<(), ()> {
    let peer_addr = conn.peer_addr().unwrap();
    let handshake_bytes = serialize_peer_handshake(handshake);

    debug!(
        "Serialized these handshake bytes: {:?}",
        handshake_bytes.to_vec()
    );

    let maybe_write = conn.write(&handshake_bytes).await;

    if maybe_write.is_ok() {
        debug!("Sent handshake to {}!", peer_addr);

        // Check to see if we got a handshake back?
        let mut buffer = vec![0x0; HANDSHAKE_BYTE_SIZE];
        let maybe_read = conn.read_exact(&mut buffer).await;

        match maybe_read {
            Ok(byte_count) => debug!("Read in {} bytes from peer {}!", byte_count, peer_addr),
            Err(_) => {
                error!(
                    "Failed to read handshake from peer {}! Error: {:?}",
                    peer_addr,
                    maybe_read.err()
                );
                return Err(());
            }
        }
        debug!("Got these bytes from peer {}: {:?}", peer_addr, buffer);

        // Verify data that we got back...
        let received_handshake = deserialize_peer_handshake(&buffer[..]);
        if received_handshake.metainfo_hash_bytes != handshake.metainfo_hash_bytes {
            error!("Metainfo from {} does not match our metainfo!", peer_addr);
            return Err(());
        }

        debug!("Metainfo matches!");
        return Ok(());
    }

    warn!(
        "Failed to send peer handshake to {}! Error: {:?}",
        peer_addr,
        maybe_write.err()
    );
    return Err(());
}

pub async fn try_establish(
    peer: &dyn ConnectablePeer,
    metainfo: &TorrentMetainfo,
) -> Result<TcpStream, ()> {
    let mut conn = try_connect(peer).await?;
    let our_handshake = gen_peer_handshake(gen_peer_id_bytes(), metainfo.gen_info_hash_bytes());
    try_handshake(&mut conn, &our_handshake).await?;
    return Ok(conn);
}
