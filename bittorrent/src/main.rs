extern crate hyper;
extern crate serde;
extern crate serde_bencode;

mod p2p;
mod torrent;

use bytes::BufMut;
use futures::future;
use hyper::body::HttpBody;
use hyper::Client;
use p2p::{connect_peer, ConnectablePeer};
use std::fs;
use tokio::prelude::*;
use torrent::{TorrentMetainfo, TrackerGetResponse};

fn truncate_or_pad(candidate: &str, width: usize) -> String {
    if candidate.len() > width {
        return String::from(&candidate[0..width]);
    } else {
        return format!("{:0>width$}", candidate, width = width);
    }
}

fn gen_peer_id() -> String {
    return truncate_or_pad("animate-test", 20);
}

fn gen_announce_get_uri(metainfo: &TorrentMetainfo) -> String {
    return format!(
        "{}?peer_id={}&info_hash={}&port={}&left={}&downloaded={}&uploaded={}&compact=1",
        metainfo.announce,
        gen_peer_id(),
        metainfo.gen_info_hash(),
        6881,
        100,
        0,
        0
    );
}

fn gen_peer_handshake(metainfo: &TorrentMetainfo) -> Vec<u8> {
    let mut buffer = vec![];
    
    buffer.push(19 as u8);
    buffer.extend(b"BitTorrent protocol");
    buffer.extend(&[0, 0, 0, 0, 0, 0, 0, 0]);
    buffer.extend(&metainfo.gen_info_hash_bytes());
    buffer.extend(gen_peer_id().as_bytes());

    return buffer;
}

async fn try_connect(peer: &dyn ConnectablePeer, metainfo: &TorrentMetainfo) {
    let maybe_connection = connect_peer(peer).await;
    match maybe_connection {
        Ok(mut conn) => {
            // Dangerous unwrap should actually handle this...
            conn.stream.write(&gen_peer_handshake(metainfo)[..]).await.unwrap();
            println!("Connected to: {}:{}!", peer.ip(), peer.port());
        }
        Err(e) => eprintln!(
            "Failed connection! To: {}:{} / {:?}",
            peer.ip(),
            peer.port(),
            e
        ),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_contents: Vec<u8> = fs::read("/Users/mana/Downloads/sao.torrent")?;
    let info: TorrentMetainfo = serde_bencode::from_bytes(&file_contents)?;
    let client = Client::new();
    let uri = gen_announce_get_uri(&info).parse()?;

    println!("Generated this URL: {}", uri);

    let mut resp = client.get(uri).await?;

    println!("Response　Status: {}", resp.status());

    let mut response_buf = vec![];

    while let Some(chunk) = resp.body_mut().data().await {
        response_buf.put_slice(&chunk?[..]);
    }

    println!("{:?}", response_buf);

    let tracker_response: TrackerGetResponse = serde_bencode::from_bytes(&response_buf)?;
    println!("{:?}", tracker_response);
    println!("{:?}", tracker_response.get_peers());

    match tracker_response.get_peers() {
        Some(peer_vec) => {
            let mut future_vec = vec![];

            for x in 0..peer_vec.len() {
                future_vec.push(try_connect(&peer_vec[x], &info));
            }

            future::join_all(future_vec).await;
        }
        None => eprintln!("Found no peers :("),
    }

    return Ok(());
}
