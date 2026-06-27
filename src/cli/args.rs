use clap::Parser;

use crate::logger::types::level::LogLevel;

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryMonitor {
    Info,
    Users,
    Vars,
}

impl QueryMonitor {
    pub fn to_str(&self) -> &str {
        match self {
            QueryMonitor::Info => "info",
            QueryMonitor::Users => "users",
            QueryMonitor::Vars => "vars",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "info" => Some(QueryMonitor::Info),
            "users" => Some(QueryMonitor::Users),
            "vars" => Some(QueryMonitor::Vars),
            _ => None,
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(version, about = "A terminal-based server monitoring tool.")]
pub struct Args {
    #[arg(
        short = 's',
        long = "store",
        help = "Storage mode (default SQLITE).",
        default_value = "sqlite"
    )]
    pub store: String,

    #[arg(
        short = 'p',
        long = "store-path",
        help = "The storage file path without the extension (default ~/.config/srv-mon-term/store{.db,.json}).",
        default_value = "~/.config/srv-mon-term/store"
    )]
    pub store_path: String,

    #[arg(
        short = 'l',
        long = "log",
        help = "Log levels to use (default: info,warn,error,fatal).",
        default_value = "info,warn,error,fatal"
    )]
    pub log: String,

    #[arg(
        short = 'L',
        long = "log-path",
        help = "Path to a log file to write logs to (default: logs/%Y-%m-%d.log)."
    )]
    pub log_path: Option<String>,

    #[arg(
        short = 'b',
        long = "basic",
        help = "Disables the advanced TUI and outputs basic text to the terminal instead.",
        default_value_t = false
    )]
    pub basic: bool,

    // Server query overrides.
    #[arg(
        short = 'd',
        long = "dst",
        help = "Destination for the server to monitor (e.g. IP:PORT)."
    )]
    pub dst: Option<String>,

    #[arg(short = 'P', long = "port", help = "Port for the server to monitor.")]
    pub port: Option<u16>,

    #[arg(short = 'q', long = "query", help = "The query type to use.")]
    pub query: Option<String>,

    #[arg(
        short = 'Q',
        long = "query-port",
        help = "The query port to use (if different from the server port)."
    )]
    pub query_port: Option<u16>,

    #[arg(
        short = 'm',
        long = "monitor-only",
        help = "When set, only monitors the specified query type and does not perform any other actions."
    )]
    pub use_query_monitor_only: bool,

    #[arg(
        short = 'M',
        long = "query-monitor",
        help = "The specific query to monitor when in basic mode.",
        default_value = "info"
    )]
    pub query_monitor: String,

    #[arg(
        short = 't',
        long = "timeout",
        help = "The timeout in seconds for server queries (default: 5)."
    )]
    pub timeout: Option<u64>,

    #[arg(
        short = 'I',
        long = "isolate",
        help = "When set along with the destination IP,port, and query, launches the program and only queries the specified server with no other options."
    )]
    pub isolate: bool,

    // Server store settings.
    #[arg(
        short = 'A',
        long = "add",
        help = "Whether to add the server to the store."
    )]
    pub add: bool,

    #[arg(
        short = 'D',
        long = "delete",
        help = "Whether to delete the server from the store."
    )]
    pub delete: bool,
}

impl Args {
    pub fn parse_log_levels(&self) -> Vec<LogLevel> {
        let levels_str = self
            .log
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        levels_str
            .map(|s| LogLevel::from_str(s).unwrap_or_default())
            .collect()
    }

    pub fn parse_query_monitor(&self) -> Option<QueryMonitor> {
        QueryMonitor::from_str(&self.query_monitor)
    }
}
