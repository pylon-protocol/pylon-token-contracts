use cosmwasm_std::{to_binary, Binary, Deps};
use cw2::get_contract_version;
use pylon_token::gov_resp::APIVersionResponse;

use crate::constant::API_VERSION;
use crate::error::ContractError;

pub mod airdrop;
pub mod bank;
pub mod config;
pub mod poll;
pub mod state;

pub type QueryResult = Result<Binary, ContractError>;

pub fn query_api_version(deps: Deps) -> QueryResult {
    Ok(to_binary(&APIVersionResponse {
        version: get_contract_version(deps.storage).unwrap().version,
    })?)
}
