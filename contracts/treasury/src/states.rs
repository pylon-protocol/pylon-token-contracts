use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub mine: Addr,
    pub controller: Addr,
    pub gas_reserve: Uint128,
    pub pylon_collector: Addr,
    pub pylon_governance: Addr,
    pub anchor_moneymarket: Addr,
    pub astroport_pair: Addr,
    pub astroport_generator: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
