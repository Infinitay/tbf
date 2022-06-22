use clap::{Parser, Subcommand};

#[derive(Debug, Clone)]
pub struct Flags {
    pub verbose: bool,
    pub simple: bool,
    pub pbar: bool,
    pub cdnfile: Option<String>,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Provide minimal output
    #[clap(short, long)]
    pub simple: bool,

    /// Show more info
    #[clap(short, long)]
    pub verbose: bool,

    /// Import more CDN urls via a config file (TXT/JSON/YAML/TOML)
    #[clap(short, long)]
    pub cdnfile: Option<String>,

    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Go over a range of timestamps, looking for a usable/working m3u8 URL, and check whether the VOD is available
    Bruteforce {
        /// Enable a progress bar (the progress bar slightly slows down the processing)
        #[clap(short, long)]
        progressbar: bool,

        /// Streamer's username (string)
        username: String,

        /// VOD/broadcast ID (integer)
        id: i64,

        /// First timestamp - either an integer (Unix time or whatever the fuck Twitch was using before) or a string (can be like "2020-11-12 20:02:13" or RFC 3339)
        from: String,

        /// Last timestamp - either an integer (Unix time or whatever the fuck Twitch was using before) or a string (can be like "2020-11-12 20:02:13" or RFC 3339)
        to: String,
    },

    /// Combine all the parts (streamer's username, VOD/broadcast ID and a timestamp) into a proper m3u8 URL and check whether the VOD is available
    Exact {
        /// Enable a progress bar (the progress bar slightly slows down the processing)
        #[clap(short, long)]
        progressbar: bool,

        /// Streamer's username (string)
        username: String,

        /// VOD/broadcast ID (integer)
        id: i64,

        /// A timestamp - either an integer (Unix time or whatever the fuck Twitch was using before) or a string (can be like "2020-11-12 20:02:13" or RFC 3339)
        stamp: String,
    },

    /// Get the m3u8 from a TwitchTracker URL
    Link {
        /// Enable a progress bar (the progress bar slightly slows down the processing)
        #[clap(short, long)]
        progressbar: bool,

        /// TwitchTracker URL
        url: String,
    },

    /// Get the m3u8 from a clip using TwitchTracker
    Clip {
        /// Enable a progress bar (the progress bar slightly slows down the processing)
        #[clap(short, long)]
        progressbar: bool,

        /// Clip's slug
        slug: String,
    },

    /// Go over a range of timestamps, looking for clips in a VOD
    Clipforce {
        /// Enable a progress bar (the progress bar slightly slows down the processing)
        #[clap(short, long)]
        progressbar: bool,

        /// VOD/broadcast ID (integer)
        id: i64,

        /// First timestamp (integer)
        start: i64,

        /// Last timestamp (integer)
        end: i64,
    },
}