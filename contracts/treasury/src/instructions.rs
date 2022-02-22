use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub mine: String,
    pub controller: String,
    pub gas_reserve: Uint128,
    pub pylon_collector: String,
    pub pylon_governance: String,
    pub anchor_moneymarket: String,
    pub astroport_pair: String,
    pub astroport_generator: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Harvest {},
    HarvestInternal {},

    // step 1: deposit ust to anchor
    StrategyAnchor { amount: Uint128 },

    // step 2: provide mine-ust liquidity
    StrategyProvideLiquidity { amount: Uint128 },

    // step 3: run buyback logic
    StrategyBuyback { amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
