extern crate hyper;
extern crate serde;
extern crate serde_bencode;

mod torrent;

use hyper::Client;
use std::fs;
use torrent::{TorrentMetainfo, TrackerGetResponse};
use hyper::body::HttpBody;
use bytes::BufMut;

fn truncate_or_pad(candidate: &str, width: usize) -> String {
    if candidate.len() > width {
        return String::from(&candidate[0..width]);
    } else {
        return format!("{:0>width$}", candidate, width = width);
    }
}


fn gen_announce_get_uri(metainfo: &TorrentMetainfo) -> String {
    return format!(
        "{}?peer_id={}&info_hash={}&port={}&left={}&downloaded={}&uploaded={}&compact=1",
        metainfo.announce,
        truncate_or_pad("animate-test", 20),
        metainfo.gen_info_hash(),
        6881,
        100,
        0,
        0
    );
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_contents: Vec<u8> = fs::read("/Users/mana/Downloads/sao.torrent").unwrap();
    let info: TorrentMetainfo = serde_bencode::from_bytes(&file_contents).unwrap();
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

    let tracker_response: TrackerGetResponse = serde_bencode::from_bytes(&response_buf).unwrap();
    println!("{:?}", tracker_response);
    println!("{:?}", tracker_response.get_peers());

    return Ok(());
}