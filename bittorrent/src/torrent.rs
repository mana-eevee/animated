struct File {
    /*
     * For the purposes of the other keys, the multi-file case is treated as
     * only having a single file by concatenating the files in the order they
     * appear in the files list. The files list is the value files maps to,
     * and is a list of dictionaries containing the following keys:
     */

    /*
     * The length of the file, in bytes.
     */
    length: u32,
    /*
     * A list of UTF-8 encoded strings corresponding to subdirectory names,
     * the last of which is the actual file name (a zero length list is an error case).
     */
    path: Vec<String>,
}

struct TorrentInfo {
    /*
     * The name key maps to a UTF-8 encoded string which is the suggested
     * name to save the file (or directory) as. It is purely advisory.
     * 
     * In the single file case, the name key is the name of a file,
     * in the muliple file case, it's the name of a directory.
     */
    name: String,
    /*
     * piece length maps to the number of bytes in each piece the file is
     * split into. For the purposes of transfer, files are split into fixed
     * size pieces which are all the same length except for possibly the
     * last one which may be truncated. piece length is almost always a
     * power of two, most commonly 2 18 = 256 K (BitTorrent prior to
     * version 3.2 uses 2 20 = 1 M as default).
     */
    piece_length: u32,
    /*
     * pieces maps to a string whose length is a multiple of 20. It
     * is to be subdivided into strings of length 20, each of which is
     * the SHA1 hash of the piece at the corresponding index.
     */
    pieces: Vec<[char, 20]>,
    /*
     * There is also a key length or a key files, but not both or neither.
     * If length is present then the download represents a single file,
     * otherwise it represents a set of files which go in a directory structure.
     * 
     * In the single file case, length maps to the length of the file in bytes.
     */
    length: u32,
    files: Vec<File>,
}

struct TorrentMetainfo {
    /*
     * The URL of the tracker.
     * */
    announce: String,
    info: TorrentInfo,
}

struct TrackerGetRequest {
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
    info_hash: [u8, 20],
    /*
     * A string of length 20 which this downloader uses as its id. Each downloader
     * generates its own id at random at the start of a new download. This value
     * will also almost certainly have to be escaped.
     * 
     * See https://www.bittorrent.org/beps/bep_0020.html for ID conventions.
     */
    peer_id: [char, 20],
    /*
     * The port number this peer is listening on. Common behavior is for a downloader
     * to try to listen on port 6881 and if that port is taken try 6882, then
     * 6883, etc. and give up after 6889.
     */
    port: u16,
    /*
     * The total amount uploaded so far, encoded in base ten ascii.
     */
    uploaded: u64,
    /*
     * The total amount downloaded so far, encoded in base ten ascii.
     */
    downloaded: u64,
    /*
     * The number of bytes this peer still has to download, encoded in base ten ascii.
     * Note that this can't be computed from downloaded and the file length since it
     * might be a resume, and there's a chance that some of the downloaded data failed
     * an integrity check and had to be re-downloaded.
     */
    left: u64,
}

struct TorrentPeer {
    peer_id: [char, 20],
    ip: String,
    port: u16,
}

struct CompactTorrentPeer {
    /*
     * 32-bit IPv4 address are 4 chunks of 1 byte each.
     */
    ip: [u8, 4],
    port: u16,
}

struct CompactTorrentPeer6 {
    /*
     * IPv6 address encoded as 8 chunks of 2 bytes each.
     */
    ip: [[u8, 2], 8],
    port: u16,
}

struct TrackerGetResponse {
    /*
     * A human readable string containing a failure reason.
     */
    failure_reason: String,
    /*
     * The number of seconds to wait before the next peer request.
     * Note that downloaders may rerequest on nonscheduled times if an event happens
     * or they need more peers.
     */
    interval: u32,
    /*
     * A list of peers for a torrent.
     */
    peers: Vec<TorrentPeer>,
    /*
     * A list of peers for a torrent when under IPv6 and using compact peer format.
     * See http://bittorrent.org/beps/bep_0007.html for details.
     */
    peers6: Vec<CompactTorrentPeer6>,
}