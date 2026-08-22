use clap::Parser;

use crate::logger::types::level::LogLevel;
use crate::server::types::query::ServerQueryType;

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
        help = "The storage file path without the extension (default ~/.config/gmon/store{.db,.json}).",
        default_value = "~/.config/gmon/store"
    )]
    pub store_path: String,

    #[arg(
        short = 'l',
        long = "log",
        help = "Log levels to use (default: info,warn,error,fatal)."
    )]
    pub log_levels: Option<String>,

    #[arg(
        short = 'L',
        long = "log-path",
        help = "Path to a log file to write logs to (default: logs/%Y-%m-%d.log)."
    )]
    pub log_path: Option<String>,

    #[arg(
        short = 'B',
        long = "log-buffer-size",
        help = "Overrides the maximum number of log messages to buffer in memory."
    )]
    pub log_max_buffer_size: Option<usize>,

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

    #[arg(
        short = 'q',
        long = "query",
        help = "The query type to use (see --list-query-types). Guessed from the port when omitted."
    )]
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
        help = "The timeout in milliseconds for server queries (default: 2000)."
    )]
    pub timeout: Option<u64>,

    #[arg(
        short = 'c',
        long = "query-interval",
        help = "How often to query the server in milliseconds (default: 1000)."
    )]
    pub query_interval: Option<u64>,

    #[arg(
        short = 'n',
        long = "name",
        help = "A display name to show for the server instead of its hostname."
    )]
    pub name: Option<String>,

    #[arg(
        long = "latency-type",
        help = "How latency is measured: self-info, self-users, self-vars, query-info, query-users, query-vars or icmp."
    )]
    pub latency_type: Option<String>,

    #[arg(
        long = "latency-interval",
        help = "How often to measure latency in milliseconds (defaults to the query interval)."
    )]
    pub latency_interval: Option<u64>,

    #[arg(
        long = "list-query-types",
        help = "Lists every supported query type and exits.",
        default_value_t = false
    )]
    pub list_query_types: bool,

    #[arg(
        short = 'I',
        long = "isolate",
        help = "When set along with the destination IP,port, and query, launches the program and only queries the specified server with no other options."
    )]
    pub isolate: bool,

    // General TUI options.
    #[arg(
        short = 'i',
        long = "draw-interval",
        help = "The interval in milliseconds to redraw the TUI (default: 1000)."
    )]
    pub draw_interval: Option<u64>,

    #[arg(
        short = 'E',
        long = "input-poll-interval",
        help = "The interval in milliseconds to poll for input in the TUI (default: 1000)."
    )]
    pub input_poll_interval: Option<u64>,

    // Server store settings.
    #[arg(
        short = 'S',
        long = "save",
        help = "Whether to add/save the server or data/settings to the store."
    )]
    pub save: bool,

    #[arg(
        short = 'D',
        long = "delete",
        help = "Whether to delete the server or data/settings from the store."
    )]
    pub delete: bool,
}

impl Args {
    pub fn parse_log_levels(levels: String) -> Vec<LogLevel> {
        let levels_str = levels
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

    /// Prints every supported query type along with the games it covers.
    pub fn print_query_types() {
        println!("Supported query types:");

        for query_type in ServerQueryType::ALL {
            let aliases = query_type.aliases().join(", ");

            println!(
                "  {:<10} {}{}",
                query_type.name(),
                query_type.description(),
                if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases)
                }
            );
        }
    }
}
