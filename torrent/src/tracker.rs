use std::net::{Ipv4Addr, SocketAddrV4};

use rand::Rng;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize, Serializer,
};
use std::fmt;

use crate::{bool_from_int, meta_info::MetaInfo};

// [u8; 20]
pub fn random_peer_id() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 20] = rng.gen();
    let s = std::str::from_utf8(&random_bytes).expect("msg");
    s.to_owned()
}

pub struct Tracker;

impl Tracker {
    pub fn request(torrent: &MetaInfo) -> Result<TrackerResponse, ()> {
        let request = TrackerRequest::new_compact(torrent);

        let query_params =
            serde_urlencoded::to_string(request).expect("failed to urlencode TrackerRequest");

        let tracker_url = torrent.tracker_url();

        let Ok(mut url) = reqwest::Url::parse(&tracker_url) else {
            return Err(());
        };

        url.set_query(Some(&query_params));

        let Ok(response) = reqwest::blocking::get(url) else {
            return Err(());
        };

        let Ok(body) = response.bytes() else {
            return Err(());
        };

        let res: TrackerResponse = serde_bencode::from_bytes(&body).expect("msg");

        Ok(res)
    }
}

#[derive(serde::Deserialize, Serialize)]
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
    info_hash: String,
    /// A string of length 20 which this downloader uses as its id.
    /// Each downloader generates its own id at random at the start of a
    /// new download. This value will also almost certainly have to be escaped. [u8; 20]
    peer_id: String,
    /// An optional parameter giving the IP (or dns name) which this peer is at.
    /// Generally used for the origin if it's on the same machine as the tracker.
    ip: Option<String>,
    /// The port number this peer is listening on.
    ///
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
    /// default=1
    #[serde(deserialize_with = "bool_from_int")]
    compact: bool,
    // This is an optional key which maps to started, completed, or stopped
    // (or empty, which is the same as not being present).
    // If not present, this is one of the announcements done at regular intervals.
    // An announcement using started is sent when a download first begins,
    // and one using completed is sent when the download is complete.
    // No completed is sent if the file was complete when started.
    // Downloaders send an announcement using stopped when they cease downloading.
    // event: String,
}

impl TrackerRequest {
    pub fn new_compact(meta_info: &MetaInfo) -> Self {
        let b = meta_info.info().hash().bytes();
        let b: &[u8] = &b;

        let info_hash =
            serde_urlencoded::from_bytes(b).expect("failed to urlencode info_hash bytes");

        Self {
            info_hash,
            peer_id: String::from("20129487650173049587"),
            port: 6881,
            ip: None,
            uploaded: 0,
            downloaded: 0,
            left: meta_info.len(),
            compact: true,
        }
    }
}

/**
    Tracker responses are bencoded dictionaries.

    If a tracker response has a key `failure reason`, then that maps to a human
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
#[serde(untagged)]
pub enum TrackerResponse {
    Success(TrackerPeerResponse),
    Failure(TrackerFailureResponse),
}

#[derive(serde::Deserialize)]
pub struct TrackerFailureResponse {
    #[serde(rename = "failure reason")]
    pub failure_reason: String,
}

#[derive(serde::Deserialize)]
pub struct TrackerPeerResponse {
    /// The number of seconds the downloader should wait between regular rerequests
    interval: usize,
    /// list of dictionaries corresponding to peers
    peers: Peers,
    // More commonly is that trackers return a compact representation of the peer list, see BEP 23.

    // If you want to make any extensions to metainfo files or tracker queries,
    // please coordinate with Bram Cohen to make sure that all extensions are done compatibly.

    // It is common to announce over a UDP tracker protocol as well.
}

impl TrackerPeerResponse {
    pub fn peers(&self) -> &Vec<SocketAddrV4> {
        &self.peers.0
    }
}

pub struct Peers(pub Vec<SocketAddrV4>);
pub struct PeersVisitor;

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = Peers;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("6 bytes per peer: 4 bytes for IPv4 address and 2 bytes for port")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() % 6 != 0 {
            return Err(E::custom(format!("invalid length: {}", v.len())));
        }

        // Preallocate Vec with the expected capacity
        let mut peers = Vec::with_capacity(v.len() / 6);
        for chunk in v.chunks_exact(6) {
            let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            peers.push(SocketAddrV4::new(ip, port));
        }

        Ok(Peers(peers))
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
        // Preallocate Vec with the exact number of bytes needed
        let mut slice = Vec::with_capacity(6 * self.0.len());
        for peer in &self.0 {
            slice.extend_from_slice(&peer.ip().octets());
            slice.extend_from_slice(&peer.port().to_be_bytes());
        }
        serializer.serialize_bytes(&slice)
    }
}
