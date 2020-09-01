use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(Serialize, Deserialize, Debug)]
pub struct PeerHandshake {
    // This is really the magic string byte count in hex.
    // The magic string is 19 bytes so 0x13 in hex.
    pub magic: u8,
    pub more_magic: [u8; 19],
    pub reserved_bytes: [u8; 8],
    pub metainfo_hash_bytes: [u8; 20],
    pub peer_id_bytes: [u8; 20],
}

pub const HANDSHAKE_BYTE_SIZE: usize = size_of::<PeerHandshake>();

pub fn gen_peer_handshake(peer_id_bytes: [u8; 20], metainfo_hash_bytes: [u8; 20]) -> PeerHandshake {
    return PeerHandshake {
        magic: 0x13,
        more_magic: *b"BitTorrent protocol",
        reserved_bytes: [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0],
        metainfo_hash_bytes: metainfo_hash_bytes,
        peer_id_bytes: peer_id_bytes,
    };
}

pub fn serialize_peer_handshake(handshake: &PeerHandshake) -> [u8; HANDSHAKE_BYTE_SIZE] {
    let byte_vec = bincode::serialize(&handshake).unwrap();
    let mut byte_array: [u8; HANDSHAKE_BYTE_SIZE] = [0x0; HANDSHAKE_BYTE_SIZE];
    byte_array.copy_from_slice(&byte_vec);
    return byte_array;
}

pub fn deserialize_peer_handshake(bytes: &[u8]) -> PeerHandshake {
    return bincode::deserialize(bytes).unwrap();
}
