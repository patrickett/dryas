//   #   | name            | status      | down         | up         | done | seeders | peers | ratio
// 10001 | ubuntu.iso      | downloading | 595.6 KiB/s  | 12.3 KiB/s | 55%  | 27 (80) | 5 (8) | 0.6
// 10002 | arch.iso        | complete    |              |            | 100% |         |       | 2.0
//

pub enum TorrentStatus {
    /// The torrent has not finished downloading
    Paused,
    /// When we still have parts of the file to download.
    Downloading,
    /// This is when the download has finished and we are now just uploading.
    Seeding,
    /// This is when the desired ratio has been hit and the download finished it stops all network traffic.
    Completed,
}

pub struct TorrentInfo {
    id: usize,
    name: String,
    status: TorrentStatus,
    download_speed: usize,
    upload_speed: usize,
    percent_done: f32,
    /// (active, total)
    seeders: (usize, usize),
    /// (active, total)
    peers: (usize, usize),

    ratio: f32,
}
