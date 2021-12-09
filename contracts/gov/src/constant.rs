pub const CONTRACT_NAME: &str = "crates.io:pylon-gov";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const POLL_EXECUTE_REPLY_ID: u64 = 1;

// pagination
pub const MAX_QUERY_LIMIT: u32 = 100;
pub const DEFAULT_QUERY_LIMIT: u32 = 50;
