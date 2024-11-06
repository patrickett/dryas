use clap::{Parser, Subcommand};
use std::path::PathBuf;
use torrent::Torrent;

/// A CLI/TUI for interacting with torrents.
///
/// https://github.com/patrickett/flud
#[derive(Parser)]
#[clap(version)]
struct Args {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Clone)]
pub enum MagnetLinkOrFilePath {
    MagnetLink(String),
    TorrentFilePath(PathBuf),
}

#[derive(Subcommand)]
enum DaemonCommands {
    /// Starts the flud daemon. This will be killed when the shell is closed or
    /// you kill the process with ctrl+c.
    ///
    /// If you want it to last beyond the shell look at... TODO:
    ///
    /// starting with systemd etc
    Start {},
    /// Accepts both magnet links as well as paths to torrent files.
    ///
    /// Will tell the daemon to add the provided magnet link
    /// or torrent file to its internal list of torrents.
    Add {
        /// You can provide either a magnet link or the path to a torrent file.
        torrent: String,

        /// Optionally set the port for where the flud daemon is listening.
        ///
        /// Defaults to `1337`
        #[clap(short = 'p', long)]
        daemon_port: Option<u16>,

        /// If not told otherwise, flud writes download torrent data to `/Downloads`.
        /// It can be instructed instead to save that data to a custom location using `-o` or `--output`q
        #[clap(short, long)]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum Command {
    /// Open a standalone TUI terminal torrent client.
    ///
    /// Things will only download while this is open.
    /// To have this run as a background process look into the flud daemon.
    // TODO: internally we might just run our own instance of the flud demon and connect to it as
    // if it was opened from this command. Then we can use the same logic for if it was ran as just a client
    // or as if it was a daemon with a client attached to it
    Open,
    /// Interact with the flud daemon/background process.
    ///
    /// If no subcommand arguments are provided open a terminal ui
    /// to let you see what the daemon is doing.
    Daemon {
        /// Optionally set the port for where the flud daemon is listening
        ///
        /// Defaults to `1337`
        #[clap(short, long)]
        port: Option<u16>,

        #[command(subcommand)]
        daemon_command: Option<DaemonCommands>,
    },
    Info {
        /// You can provide a path to a torrent file.
        path: PathBuf,
    },

    /// Start downloading the provided magnet link or torrent file path
    Download {
        /// You can provide either a magnet link or the path to a torrent file.
        torrent: String,
    },
}

fn main() {
    let args = Args::parse();

    if let Some(command) = args.cmd {
        match command {
            Command::Open => todo!(),
            Command::Daemon {
                port: _,
                daemon_command,
            } => {
                if let Some(_d_command) = daemon_command {
                    todo!("run some command for the flud daemon")
                } else {
                    todo!("open tui while connecting to the flud daemon")
                }
            }
            Command::Download { torrent: _ } => {
                // allow ctrl+c to cancel and picking back up if reran
                todo!()
            }
            Command::Info { path } => {
                if let Ok(torrent) = Torrent::try_from(path) {
                    println!("info hash: {}", torrent.info().hash());
                    println!("piece length: {}", torrent.info().piece_length());
                    // println!("piece hashes:");

                    // for hash in torrent.info().pieces() {
                    //     println!("{}", hex::encode(hash))
                    // }
                } else {
                    eprintln!("unable to parse torrent file")
                }
            }
        }
    } else {
        todo!()
        // open_standalone_tui()
    }
}

// All, Downloading, Seeding, Active, Paused, Complete
// Tags?

// '/' to search depending on the selected tab
// rss feeds
// search dht
// check command to check file against torrent
// create torrent
// labels or tags

// alternative_names: fld;flud;

// add option to have a compact mode and a cozy mode
// cozy = fancy ui stuffs like progress bar
// compact is single line per torrent

// [########################################################                                     ]

// ~/.flud/settings.toml (this is what the settings tab edits)

// instead of a database we have a folder based state with .torrent files
// ~/.flud/downloading
// ~/.flud/paused
// ~/.flud/seeding
// ~/.flud/completed

// downloading->(optional paused)->seeding->completed

// so only downloading and seeding folders actually have actionable items
// seeding: wait for ratio then move to completed
// downloading: wait until finished downloading then move to seeding
// paused and completed are just for listing

// TODO: if we move it back from completed to seeding to continue after ratio is hit we need
// a `ignore_ratio` bool

// maybe instead of inline if we ask to download, it adds them to a db or store of some kind
// so that we can see them when we open the client. and all the daemon does is exactly what the
// client does but constantly without showing anything
//
// flud add "magnet?=asd" -> creates .torrent in /downloading ->
