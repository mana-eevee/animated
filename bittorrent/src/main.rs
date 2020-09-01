extern crate hyper;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_bencode;
extern crate stderrlog;

mod client;
mod p2p;
mod torrent;

use bytes::BufMut;
use client::try_connect;
use futures::future;
use hyper::body::HttpBody;
use hyper::Client;
use std::fs;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    stderrlog::new()
        .module(module_path!())
        .verbosity(4)
        .color(stderrlog::ColorChoice::Always)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();

    let file_contents: Vec<u8> = fs::read("/home/mana/Downloads/sao.torrent")?;
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
