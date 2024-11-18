use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    Deserialize, Serialize, Serializer,
};
use std::{fmt, path::PathBuf};

// https://www.bittorrent.org/beps/bep_0003.html
// https://wiki.theory.org/BitTorrentSpecification#Metainfo_File_Structure

// TODO: unit tests
// TODO: is it worth our own bencode impl for speed?

/// MetaInfo files (also known as .torrent files) are bencoded dictionaries.
/// All strings in a .torrent file that contains text must be UTF-8 encoded.
#[derive(Debug, Deserialize, Serialize)]
pub struct MetaInfo {
    /// a dictionary that describes the file(s) of the torrent.
    /// There are two possible forms: one for the case of a 'single-file' torrent
    ///  with no directory structure, and one for the case of a 'multi-file' torrent
    info: Info,
    /// The announce URL of the tracker (string)
    announce: String,
    /// (optional) this is an extention to the official specification, offering backwards-compatibility. (list of lists of strings).
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    /// (optional) the creation time of the torrent, in standard UNIX epoch format (integer, seconds since 1-Jan-1970 00:00:00 UTC)
    #[serde(rename = "creation date")]
    creation_date: Option<u64>,
    /// (optional) free-form textual comments of the author (string)
    comment: Option<String>,
    /// (optional) name and version of the program used to create the .torrent (string)
    #[serde(rename = "created by")]
    created_by: Option<String>,
    /// (optional) the string encoding format used to generate the pieces part of the info dictionary in the .torrent metafile (string)
    encoding: Option<String>,
}

pub enum MetaInfoError {
    InvalidPath,
    UnableToReadFile,
    BencodeParseFailed,
}

impl TryFrom<PathBuf> for MetaInfo {
    type Error = MetaInfoError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        if !path.exists() {
            return Err(MetaInfoError::InvalidPath);
        }

        let Ok(torrent_file_bytes) = std::fs::read(path) else {
            return Err(MetaInfoError::UnableToReadFile);
        };

        match serde_bencode::from_bytes(&torrent_file_bytes) {
            Ok(meta_info) => Ok(meta_info),
            Err(err) => {
                eprintln!("{:#?}", err);
                Err(MetaInfoError::BencodeParseFailed)
            }
        }
    }
}

impl MetaInfo {
    pub fn info(&self) -> &Info {
        &self.info
    }

    pub fn tracker_url(&self) -> &str {
        &self.announce
    }

    /// Length of the file
    pub const fn len(&self) -> usize {
        match self.info.key {
            Key::SingleFile { length } => length,
            Key::MultiFile { files: _ } => todo!(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// TODO: should info actually be enum?
// enum Info {SingleFile, MultiFile}
#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    /// The name key maps to a UTF-8 encoded string which is the suggested name
    /// to save the file (or directory) as. It is purely advisory.
    ///
    /// In the single file case, the name key is the name of a file,
    /// in the muliple file case, it's the name of a directory.
    name: String,
    /// piece length maps to the number of bytes in each piece the file is split
    /// into. For the purposes of transfer, files are split into fixed-size
    /// pieces which are all the same length except for possibly the last one
    /// which may be truncated. piece length is almost always a power of two, most
    /// commonly 2 18 = 256 K (BitTorrent prior to version 3.2 uses 2 20 = 1 M as default).
    #[serde(rename = "piece length")]
    piece_length: usize,
    /// pieces maps to a string whose length is a multiple of 20.
    /// It is to be subdivided into strings of length 20, each of which
    /// is the SHA1 hash of the piece at the corresponding index.
    pieces: Hashes,

    // #[serde(deserialize_with = "bool_from_optional_int")]
    private: Option<u8>,

    #[serde(flatten)]
    key: Key,
}

impl Info {
    pub fn private(&self) -> bool {
        match self.private {
            Some(num) => match num {
                0 => false,
                // Since they declared private and its not false
                // then its safe to assume they want it private
                _ => true,
            },
            None => false,
        }
    }

    pub fn pieces(&self) -> &Vec<[u8; 20]> {
        &self.pieces.0
    }

    pub fn piece_length(&self) -> usize {
        self.piece_length
    }

    pub fn hash(&self) -> sha1_smol::Digest {
        let bencoded_info = serde_bencode::to_bytes(&self).expect("failed to bencode info");
        let mut m = sha1_smol::Sha1::new();
        m.update(&bencoded_info);

        m.digest()
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(KeysVisitor)
    }
}

struct KeysVisitor;

impl<'de> Visitor<'de> for KeysVisitor {
    type Value = Key;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map with either a `length` or `files` field")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Temporary storage for fields
        let mut length: Option<usize> = None;
        let mut files: Option<Vec<File>> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "length" => {
                    if length.is_some() {
                        return Err(de::Error::duplicate_field("length"));
                    }
                    length = Some(map.next_value()?);
                }
                "files" => {
                    if files.is_some() {
                        return Err(de::Error::duplicate_field("files"));
                    }
                    files = Some(map.next_value()?);
                }
                _ => {
                    return Err(de::Error::unknown_field(&key, &["length", "files"]));
                }
            }
        }

        // Determine the variant based on which field was present
        if let Some(length) = length {
            Ok(Key::SingleFile { length })
        } else if let Some(files) = files {
            Ok(Key::MultiFile { files })
        } else {
            Err(de::Error::missing_field("length or files"))
        }
    }
}

/// There is also a key length or a key files, but not both or neither.
// NOTE: we did not use serde(untagged) for performance reasons
#[derive(Debug, Serialize)]
pub enum Key {
    /// If length is present then the download represents a single file,
    /// otherwise it represents a set of files which go in a directory structure.
    SingleFile {
        /// In the single file case, length maps to the length of the file in bytes.
        length: usize,
    },
    /// For the purposes of the other keys, the multi-file case is treated
    /// as only having a single file by concatenating the files in the order
    /// they appear in the files list.
    MultiFile {
        /// The files list is the value files maps to, and is a list of
        /// dictionaries containing the following keys:
        files: Vec<File>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    /// The length of the file, in bytes.
    length: usize,
    /// A list of UTF-8 encoded strings corresponding to subdirectory names,
    /// the last of which is the actual file name (a zero length list is an error case).
    path: Vec<String>,
    // (optional) a 32-character hexadecimal string corresponding to the MD5 sum of the file. This is not used by BitTorrent at all, but it is included by some programs for greater compatibility.
    // md5sum: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Hashes(pub Vec<[u8; 20]>);

struct HashesVisitor;

impl Visitor<'_> for HashesVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte string whose length is a multiple of 20")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let len = v.len();
        if len % 20 != 0 {
            return Err(E::custom(format!("length is {}", len)));
        }

        // Preallocate the vector with the exact required capacity
        let count = len / 20;
        let mut hashes = Vec::with_capacity(count);

        // Manually chunk the bytes into arrays of length 20
        for chunk in v.chunks_exact(20) {
            let mut array = [0u8; 20]; // Pre-allocate the array
            array.copy_from_slice(chunk); // Copy the data directly
            hashes.push(array); // Push the array into the Vec
        }

        Ok(Hashes(hashes))
    }
}

impl<'de> Deserialize<'de> for Hashes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashesVisitor)
    }
}

impl Serialize for Hashes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Use `Vec::as_slice` to avoid concatenation and create a flat slice
        let total_length = self.0.len() * 20;
        let mut output = Vec::with_capacity(total_length);
        for hash in &self.0 {
            output.extend_from_slice(hash);
        }
        serializer.serialize_bytes(&output)
    }
}
