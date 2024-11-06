use std::net::SocketAddrV4;

use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    Deserialize, Serialize, Serializer,
};
use std::fmt;

#[derive(serde::Deserialize)]
pub struct TrackerRequest {
    /// The 20 byte sha1 hash of the bencoded form of the info value from the
    /// metainfo file. This value will almost certainly have to be escaped.
    ///
    ///
    /// Note that this is a substring of the metainfo file.
    /// The info-hash must be the hash of the encoded form as found in
    /// the .torrent file, which is identical to bdecoding the metainfo file,
    /// extracting the info dictionary and encoding it if and only if the bdecoder
    /// fully validated the input (e.g. key ordering, absence of leading zeros).
    /// Conversely that means clients must either reject invalid metainfo files
    /// or extract the substring directly. They must not perform a
    /// decode-encode roundtrip on invalid data.
    info_hash: sha1_smol::Digest,
    /// A string of length 20 which this downloader uses as its id.
    /// Each downloader generates its own id at random at the start of a
    /// new download. This value will also almost certainly have to be escaped.
    peer_id: String,
    /// An optional parameter giving the IP (or dns name) which this peer is at.
    /// Generally used for the origin if it's on the same machine as the tracker.
    // ip: String,
    /// The port number this peer is listening on.
    /// Common behavior is for a downloader to try to listen on port 6881 and if
    /// that port is taken try 6882, then 6883, etc. and give up after 6889.
    port: u16,
    /// The total amount uploaded so far, encoded in base ten ascii.
    uploaded: usize,
    /// The total amount downloaded so far, encoded in base ten ascii.
    downloaded: usize,
    /// The number of bytes this peer still has to download,
    /// encoded in base ten ascii. Note that this can't be computed from
    /// downloaded and the file length since it might be a resume,
    /// and there's a chance that some of the downloaded data failed an integrity
    /// check and had to be re-downloaded.
    left: usize,
    /// https://www.bittorrent.org/beps/bep_0023.html
    #[serde(deserialize_with = "bool_from_int")]
    compact: bool, // default=1
    /// This is an optional key which maps to started, completed, or stopped
    /// (or empty, which is the same as not being present).
    /// If not present, this is one of the announcements done at regular intervals.
    /// An announcement using started is sent when a download first begins,
    /// and one using completed is sent when the download is complete.
    /// No completed is sent if the file was complete when started.
    /// Downloaders send an announcement using stopped when they cease downloading.
    event: String,
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => {
            // Err(de::Error::invalid_value(
            //     Unexpected::Unsigned(other as u64),
            //     &"zero or one",
            // ))

            todo!()
        }
    }
}

/**
    Tracker responses are bencoded dictionaries.

    If a tracker response has a key failure reason, then that maps to a human
    readable string which explains why the query failed, and no other keys are
    required.

    Otherwise, it must have two keys: `interval`, which maps to the
    number of seconds the downloader should wait between regular rerequests,
    and `peers`. `peers` maps to a list of dictionaries corresponding to peers,
    each of which contains the keys peer id, ip, and port, which map to the
    peer's self-selected ID, IP address or dns name as a string, and port number,
    respectively.

    Note that downloaders may rerequest on nonscheduled times if an event
    happens or they need more peers.
*/
#[derive(serde::Deserialize)]
pub struct TrackerResponse {
    /// The number of seconds the downloader should wait between regular rerequests
    interval: usize,
    /// list of dictionaries corresponding to peers
    peers: Peers,
    // More commonly is that trackers return a compact representation of the peer list, see BEP 23.

    // If you want to make any extensions to metainfo files or tracker queries,
    // please coordinate with Bram Cohen to make sure that all extensions are done compatibly.

    // It is common to announce over a UDP tracker protocol as well.
}

pub struct Peers(pub Vec<SocketAddrV4>);
pub struct PeersVisitor;

impl Visitor<'_> for PeersVisitor {
    type Value = Peers;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("6 bytes, first 4 are the ipv4 last 2 are port")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        todo!()
        // let len = v.len();
        // if len % 6 != 0 {
        //     return Err(E::custom(format!("length is {}", len)));
        // }

        // // Preallocate the vector with the exact required capacity
        // let count = len / 20;
        // let mut hashes = Vec::with_capacity(count);

        // // Manually chunk the bytes into arrays of length 20
        // for chunk in v.chunks_exact(20) {
        //     let mut array = [0u8; 20]; // Pre-allocate the array
        //     array.copy_from_slice(chunk); // Copy the data directly
        //     hashes.push(array); // Push the array into the Vec
        // }

        // Ok(Peers(hashes))
    }
}

impl<'de> Deserialize<'de> for Peers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}

impl Serialize for Peers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

// pub struct Peer {
//     peer_id: usize,
//     ip: String,
//     port: u16,
// }
