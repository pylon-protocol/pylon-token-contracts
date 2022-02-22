use cosmwasm_std::Binary;

use crate::error::ContractError;

pub mod airdrop;
pub mod bank;
pub mod config;
pub mod poll;
pub mod state;

pub type QueryResult = Result<Binary, ContractError>;
