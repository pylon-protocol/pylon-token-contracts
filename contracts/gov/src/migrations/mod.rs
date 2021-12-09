pub mod state;

use cosmwasm_std::Response;

use crate::error::ContractError;

pub type MigrateResult = Result<Response, ContractError>;
