use crate::{constants::ROCKSDB_PATH, structs::Anime};
use rocksdb::{IteratorMode, DB};
use std::collections::HashMap;

pub fn upsert_anime(anime: &Anime) -> String {
    let db = DB::open_default(ROCKSDB_PATH).expect("Failed to open local RocksDB to upsert anime.");

    db.put(
        anime.to_hash().as_bytes(),
        bincode::serialize(anime).unwrap(),
    )
    .expect("Failed to upsert anime in RocksDB after opening.");

    return anime.to_hash();
}

pub fn list_anime() -> HashMap<String, Anime> {
    let db = DB::open_default(ROCKSDB_PATH).expect("Failed to open local RocksDB to list anime.");
    let mut all_anime = HashMap::new();
    let iter = db.iterator(IteratorMode::Start);
    for (key, value) in iter {
        // Unbox the Box and then get a pointer.
        let key_byte_array: &[u8] = &*key;
        let anime_byte_array: &[u8] = &*value;
        let anime = bincode::deserialize::<Anime>(anime_byte_array).unwrap();

        if anime.tombstone {
            continue;
        }

        all_anime.insert(String::from_utf8(key_byte_array.to_vec()).unwrap(), anime);
    }
    return all_anime;
}
