pub enum TorrentState {
    Downloading,
    Seeding,
    /// Downloading and Uploading
    Active,
    Paused,
    Queued,
    Complete,
}
