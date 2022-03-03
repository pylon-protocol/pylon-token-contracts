use astroport::asset;
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
    /// ## Description
    /// This instruction executes strategy of Pylon Treasury.  
    ///
    /// ### Overview of execution
    /// 1. Sweeps all UST in collector contract
    /// 2. Executes [`ExecuteMsg::HarvestInternal`] message
    ///
    /// ## Example
    /// ```typescript
    /// {
    ///   "harvest": {}
    /// }
    /// ```
    Harvest {},
    /// ## Description
    /// This instruction executes internal operation of Pylon Treasury's strategy
    ///
    /// ### Overview of execution
    /// 1. Check contract balance
    /// 2. Execute strategy messages.
    ///     * Step 1: [`ExecuteMsg::StrategyAnchor`]
    ///     * Step 2: [`ExecuteMsg::StrategyProvideLiquidity`]
    ///     * Step 3: [`ExecuteMsg::StrategyBuyback`]
    ///
    /// ## Example
    /// ```typescript
    /// {
    ///   "harvest_internal": {}
    /// }
    /// ```
    HarvestInternal {},

    /// ## Description
    /// This instruction executes step 1 of strategy: Deposit ust to anchor
    ///
    /// ## Example
    /// ```typescript
    /// {
    ///   "strategy_anchor": {
    ///     "amount": "10000000" // amount in UST
    ///   }
    /// }
    /// ```
    StrategyAnchor { amount: Uint128 },

    /// ## Description
    /// This instruction executes step 2 of strategy: Provide mine-ust liquidity
    ///
    /// ## Example
    /// ```typescript
    /// {
    ///   "strategy_provide_liquidity: {
    ///     "amount": "1000000000" // amount in UST
    ///   }
    /// }
    /// ```
    StrategyProvideLiquidity { amount: Uint128 },

    /// ## Description
    /// This instruction executes step 3 of strategy: Run buyback logic
    ///
    /// ## Example
    /// ```typescript
    /// {
    ///   "strategy_buyback": {
    ///     "amount": "10000000000" // amount in UST
    ///   }
    /// }
    /// ```
    StrategyBuyback { amount: Uint128 },

    /// ## Description
    /// This instruction executes withdraw operation.
    ///
    /// ## Example
    /// ```typescript
    /// {
    ///   "withdraw": {
    ///     "info": {
    ///       // option 1: cw20
    ///       "token": {
    ///         "contract_addr": "terra1..."
    ///       }
    ///       // option 2: native
    ///       "native_token": {
    ///         "denom": "uusd"
    ///       }
    ///     },
    ///     "amount": "100000000"
    ///   }
    /// }
    /// ```
    Withdraw { target: asset::Asset },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Returns [`ConfigResponse`] which contains current state of contract's configuration
    ///
    /// ## Example
    /// ```typescript
    /// {
    ///   "config": {}
    /// }
    /// ```
    Config {},

    /// ## Description
    /// Returns [`StateResponse`] which contains current state of contract's mutable values
    ///
    /// ## Example
    /// ```typescript
    /// {
    ///   "state": {}
    /// }
    /// ```
    State {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
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
pub struct StateResponse {
    pub prev_harvest_time: u64,
    pub pending_ust: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
