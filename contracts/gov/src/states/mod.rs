pub mod airdrop;
pub mod bank;
pub mod config;
pub mod poll;
pub mod state;

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_STATE: &[u8] = b"state";
pub static KEY_TMP_POLL_ID: &[u8] = b"tmp_poll_id";

pub static PREFIX_AIRDROP: &[u8] = b"airdrop";
pub static PREFIX_AIRDROP_REWARD: &[u8] = b"airdrop_reward";
pub static PREFIX_BANK: &[u8] = b"bank";
pub static PREFIX_BANK_UNLOCK_CLAIM: &[u8] = b"bank_unlock_claim";
pub static PREFIX_POLL: &[u8] = b"poll";
pub static PREFIX_POLL_VOTER: &[u8] = b"poll_voter";
pub static PREFIX_POLL_INDEXER: &[u8] = b"poll_indexer";
pub static PREFIX_POLL_INDEXER_STATUS: &[u8] = b"status";
pub static PREFIX_POLL_INDEXER_CATEGORY: &[u8] = b"category";
