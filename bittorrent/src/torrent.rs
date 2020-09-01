use crate::client::ConnectablePeer;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::convert::TryInto;
use std::fmt::Write;
use std::mem::size_of;
use std::rc::Rc;

fn sha1_bytes_to_hex_string(bytes: &[u8; 20]) -> String {
    let mut s = String::new();

    for byte in bytes {
        write!(&mut s, "{:x}", byte).unwrap();
    }

    return s;
}

fn url_encode_bytes(bytes: &[u8]) -> String {
    let mut s = String::new();

    for byte in bytes {
        // Extra character padding is very important otherwise
        // the URL encoding is not valid.
        write!(&mut s, "%{:0>2x}", byte).unwrap();
    }

    return s;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    /*
     * For the purposes of the other keys, the multi-file case is treated as
     * only having a single file by concatenating the files in the order they
     * appear in the files list. The files list is the value files maps to,
     * and is a list of dictionaries containing the following keys:
     */

    /*
     * The length of the file, in bytes.
     */
    pub length: u64,
    /*
     * A list of UTF-8 encoded strings corresponding to subdirectory names,
     * the last of which is the actual file name (a zero length list is an error case).
     */
    pub path: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TorrentInfo {
    /*
     * The name key maps to a UTF-8 encoded string which is the suggested
     * name to save the file (or directory) as. It is purely advisory.
     *
     * In the single file case, the name key is the name of a file,
     * in the muliple file case, it's the name of a directory.
     */
    pub name: String,
    /*
     * piece length maps to the number of bytes in each piece the file is
     * split into. For the purposes of transfer, files are split into fixed
     * size pieces which are all the same length except for possibly the
     * last one which may be truncated. piece length is almost always a
     * power of two, most commonly 2 18 = 256 K (BitTorrent prior to
     * version 3.2 uses 2 20 = 1 M as default).
     */
    #[serde(rename = "piece length")]
    pub piece_length: u32,
    /*
     * pieces maps to a string whose length is a multiple of 20. It
     * is to be subdivided into strings of length 20, each of which is
     * the SHA1 hash of the piece at the corresponding index.
     */
    pub pieces: ByteBuf,
    /*
     * There is also a key length or a key files, but not both or neither.
     * If length is present then the download represents a single file,
     * otherwise it represents a set of files which go in a directory structure.
     *
     * In the single file case, length maps to the length of the file in bytes.
     */
    pub length: Option<u32>,
    pub files: Option<Vec<File>>,
}

impl TorrentInfo {
    pub fn get_piece_sha1(&self, index: u32) -> String {
        let start_index = 20 * index as usize;
        let end_index = start_index + 20 as usize;
        let sha1_bytes = self.pieces[start_index..end_index].try_into().unwrap();
        return sha1_bytes_to_hex_string(&sha1_bytes);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TorrentMetainfo {
    /*
     * The URL of the tracker.
     * */
    pub announce: String,
    pub info: TorrentInfo,
}

impl TorrentMetainfo {
    pub fn gen_info_hash_bytes(&self) -> [u8; 20] {
        let encoded_info = serde_bencode::to_bytes(&self.info).unwrap();
        let mut hasher = Sha1::new();
        hasher.update(&encoded_info);
        return hasher.finalize().to_vec()[..].try_into().unwrap();
    }

    pub fn gen_info_hash(&self) -> String {
        let info_hash = self.gen_info_hash_bytes();
        return url_encode_bytes(&info_hash[..]);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackerGetRequest {
    /*
     * The 20 byte sha1 hash of the bencoded form of the info value from the
     * metainfo file. This value will almost certainly have to be escaped.
     *
     * Note that this is a substring of the metainfo file. The info-hash must
     * be the hash of the encoded form as found in the .torrent file, which is
     * identical to bdecoding the metainfo file, extracting the info dictionary
     * and encoding it if and only if the bdecoder fully validated the input
     * (e.g. key ordering, absence of leading zeros). Conversely that means clients
     * must either reject invalid metainfo files or extract the substring directly.
     * They must not perform a decode-encode roundtrip on invalid data.
     */
    pub info_hash: ByteBuf,
    /*
     * A string of length 20 which this downloader uses as its id. Each downloader
     * generates its own id at random at the start of a new download. This value
     * will also almost certainly have to be escaped.
     *
     * See https://www.bittorrent.org/beps/bep_0020.html for ID conventions.
     */
    pub peer_id: String,
    /*
     * The port number this peer is listening on. Common behavior is for a downloader
     * to try to listen on port 6881 and if that port is taken try 6882, then
     * 6883, etc. and give up after 6889.
     *
     * This is in BIG ENDIAN.
     */
    pub port: u16,
    /*
     * The total amount uploaded so far, encoded in base ten ascii.
     */
    pub uploaded: u64,
    /*
     * The total amount downloaded so far, encoded in base ten ascii.
     */
    pub downloaded: u64,
    /*
     * The number of bytes this peer still has to download, encoded in base ten ascii.
     * Note that this can't be computed from downloaded and the file length since it
     * might be a resume, and there's a chance that some of the downloaded data failed
     * an integrity check and had to be re-downloaded.
     */
    pub left: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TorrentPeer {
    pub peer_id: String,
    /*
     * 32-bit IPv4 address are 4 chunks of 1 byte each.
     */
    pub ip: [u8; 4],
    /*
     * This is in BIG ENDIAN.
     */
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompactTorrentPeer {
    /*
     * 32-bit IPv4 address are 4 chunks of 1 byte each.
     */
    pub ip: [u8; 4],
    /*
     * This is in BIG ENDIAN.
     */
    pub port: u16,
}

impl ConnectablePeer for CompactTorrentPeer {
    fn ip(&self) -> String {
        let ip = Rc::new(&self.ip);
        let mut parts = vec![];

        for byte in *ip {
            parts.push(format!("{}", *byte as u32));
        }

        return parts[..].join(".");
    }

    fn port(&self) -> u16 {
        return self.port;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompactTorrentPeer6 {
    /*
     * IPv6 address encoded as 8 chunks of 2 bytes each.
     */
    pub ip: [[u8; 2]; 8],
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackerGetResponse {
    /*
     * A human readable string containing a failure reason.
     */
    pub failure_reason: Option<String>,
    /*
     * The number of seconds to wait before the next peer request.
     * Note that downloaders may rerequest on nonscheduled times if an event happens
     * or they need more peers.
     */
    pub interval: Option<u32>,
    /*
     * A list of peers for a torrent.
     */
    pub peers: Option<ByteBuf>,
    /*
     * A list of peers for a torrent when under IPv6 and using compact peer format.
     * See http://bittorrent.org/beps/bep_0007.html for details.
     */
    pub peers6: Option<ByteBuf>,
}

impl TrackerGetResponse {
    pub fn get_peers(&self) -> Option<Vec<CompactTorrentPeer>> {
        match &self.peers {
            Some(peer_bytes) => {
                // The peers byte buffer is a contiguous chunk
                // of bytes where each block of 6 bytes is one peer.
                // Take fixed sized slices of the buffer and deserialize
                // each block into a CompactTorrentPeer.
                let mut peers = vec![];
                let mut start = 0;
                const PEER_BYTE_SIZE: usize = size_of::<CompactTorrentPeer>();

                while start < peer_bytes.len() {
                    peers.push(CompactTorrentPeer {
                        ip: [
                            peer_bytes[start],
                            peer_bytes[start + 1],
                            peer_bytes[start + 2],
                            peer_bytes[start + 3],
                        ],
                        // Big endian decoding of the port.
                        port: ((peer_bytes[start + 4] as u16) << 8)
                            | (peer_bytes[start + 5] as u16),
                    });
                    start += PEER_BYTE_SIZE;
                }

                return Some(peers);
            }
            None => None,
        }
    }
}
