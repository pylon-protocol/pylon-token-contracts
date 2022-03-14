use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::common::OrderBy;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub voting_token: String,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
    pub unstaking_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollMsg {
    CastVote {
        poll_id: u64,
        vote: VoteOption,
        amount: Uint128,
    },
    Execute {
        poll_id: u64,
    },
    ExecuteMsgs {
        poll_id: u64,
    },
    Snapshot {
        poll_id: u64,
    },
    End {
        poll_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    StakeInternal {
        sender: String,
        amount: Uint128,
    },
    Unstake {
        amount: Option<Uint128>,
    },
    UnstakeInternal {
        sender: String,
        amount: Option<Uint128>,
    },
    Unlock {
        target: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AirdropMsg {
    Instantiate {
        start: u64,
        period: u64,
        reward_token: String,
        reward_amount: Uint128,
    },
    Allocate {
        airdrop_id: u64,
        recipient: String,
        allocate_amount: Uint128,
    },
    Deallocate {
        airdrop_id: u64,
        recipient: String,
        deallocate_amount: Uint128,
    },
    Update {
        target: Option<String>,
    },
    Claim {
        target: Option<String>,
    },
    ClaimInternal {
        sender: String,
        airdrop_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        owner: Option<String>,
        quorum: Option<Decimal>,
        threshold: Option<Decimal>,
        voting_period: Option<u64>,
        timelock_period: Option<u64>,
        proposal_deposit: Option<Uint128>,
        snapshot_period: Option<u64>,
    },
    Poll(PollMsg),
    Staking(StakingMsg),
    Airdrop(AirdropMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// StakeVotingTokens a user can stake their mirror token to receive rewards
    /// or do vote on polls
    Stake {},
    /// CreatePoll need to receive deposit from a proposer
    CreatePoll {
        title: String,
        category: PollCategory,
        description: String,
        link: Option<String>,
        execute_msgs: Option<Vec<PollExecuteMsg>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PollExecuteMsg {
    pub order: u64,
    pub contract: String,
    pub msg: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Returns [`ConfigResponse`] which contains current state of contract's configuration
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "config": {},
    /// }
    /// ```
    Config {},

    /// ## Description
    /// Returns [`StateResponse`] which contains current state of contract's mutable values
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "state": {},
    /// }
    /// ```
    ///
    State {},

    /// ## Description
    /// Returns [`ClaimResponse`] which contains registered token claim's information.
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "claim": {
    ///         "address": "terra1xu8utj38xuw6mjwck4n97enmavlv852zkcvhgp",
    ///         "claim_id": 10,
    ///     },
    /// }
    /// ```
    Claim {
        /// Address of claim's owner
        address: String,
        /// Number of claim to query
        claim_id: u64,
    },

    /// ## Description
    /// Returns [`ClaimsResponse`] which contains information of list of specific staker's token claims.
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "claims": {
    ///         "address": "terra1xu8utj38xuw6mjwck4n97enmavlv852zkcvhgp",
    ///         "start_after": 10,
    ///         "limit": 50,
    ///         "order": "asc",
    ///     },
    /// }
    /// ```
    Claims {
        /// Address of claims' owner
        address: String,
        /// Starting point of range query.
        /// default value is None. which is means the genesis
        start_after: Option<u64>,
        /// Limit parameter of range query.
        /// default value is 50.
        limit: Option<u32>,
        /// Order parameter of range query.
        /// You can choose between "asc" | "desc".
        /// default value is "asc"
        order: Option<OrderBy>,
    },

    /// ## Description
    /// Returns [`StakerResponse`] which contains staker's information of stakeholdings.
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "staker": {
    ///         "address": "terra1xu8utj38xuw6mjwck4n97enmavlv852zkcvhgp",
    ///     },
    /// }
    /// ```
    Staker {
        /// Address of staker
        address: String,
    },

    /// ## Description
    /// Returns [`StakersResponse`] which contains list of stakers' information of stakeholdings.
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "stakers": {
    ///         "start_after": "terra1xu8utj38xuw6mjwck4n97enmavlv852zkcvhgp",
    ///         "limit": 50,
    ///         "order": "asc",
    ///     },
    /// }
    /// ```
    Stakers {
        /// Starting point of range query to list of stakers
        /// default value is None. which is means the genesis
        start_after: Option<String>,
        /// Limit parameter of range query
        /// default value is 50.
        limit: Option<u32>,
        /// Order parameter of range query
        /// You can choose between "asc" | "desc"
        /// default value is "asc".
        order: Option<OrderBy>,
    },

    /// ## Description
    /// Returns [`AirdropResponse`] which contains current state of specific airdrop.
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "airdrop": {
    ///         "airdrop_id": 10,
    ///     },
    /// }
    /// ```
    Airdrop { airdrop_id: u64 },

    /// ## Description
    /// Returns [`AirdropsResponse`] which contains current state of list of airdrops.
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "airdrops": {
    ///         "start_after": 10,
    ///         "limit": 50,
    ///         "order_by": "asc" | "desc",
    ///     },
    /// }
    /// ```
    Airdrops {
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },

    /// ## Description
    /// Returns [`PollResponse`] which contains current state of registered poll.
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "poll": {
    ///         "poll_id": 12,
    ///     },
    /// }
    /// ```
    Poll { poll_id: u64 },

    /// ## Description
    /// Returns [`PollsResponse`] which contains current state of list of polls.
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "polls": {
    ///         "status_filter": "in_progress",
    ///         "category_filter": "core",
    ///         "start_after": 5,
    ///         "limit": 50,
    ///         "order_by": "asc",
    ///     },
    /// }
    /// ```
    Polls {
        status_filter: Option<PollStatus>,
        category_filter: Option<PollCategory>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },

    /// ## Description
    /// Returns [`VotersResponse`] which contains voters' information by specific poll
    ///
    /// **Example**
    /// ```typescript
    /// {
    ///     "voters": {
    ///         "poll_id": 9,
    ///         "start_after": "terra1xu8utj38xuw6mjwck4n97enmavlv852zkcvhgp",
    ///         "limit": 50,
    ///         "order_by": "asc",
    ///     },
    /// }
    /// ```
    Voters {
        poll_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableAirdrop {
    pub token: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoterInfo {
    pub vote: VoteOption,
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    InProgress,
    Passed,
    Rejected,
    Executed,
    Expired, // Deprecated
    Failed,
}

impl fmt::Display for PollStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollCategory {
    Core,
    Gateway,
    None,
}

impl fmt::Display for PollCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteOption {
    Yes,
    No,
}

impl fmt::Display for VoteOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self == VoteOption::Yes {
            write!(f, "yes")
        } else {
            write!(f, "no")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {
    State { total_unbondings: Uint128 },
    General {},
}

/// ## Description
/// Response struct of [`QueryMsg::Config`] query message
///
/// **Example**
/// ```typescript
/// {
///     "owner": "terra1xu8utj38xuw6mjwck4n97enmavlv852zkcvhgp",
///     "pylon_token": "terra1xu8utj38xuw6mjwck4n97enmavlv852zkcvhgp",
///     "quorum": "0.123",
///     "threshold": "0.3213",
///     "voting_period": "86400",
///     "timelock_period": "86400",
///     "proposal_deposit": "10000000000", // 6 decimal
///     "snapshot_period": "86400",
///     "unstaking_period": "86400",
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub pylon_token: String,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
    pub unstaking_period: u64,
}

/// ## Description
/// Response struct of [`QueryMsg::State`] query message
///
/// **Example**
/// ```typescript
/// {
///     "poll_count": 10,
///     "total_share": "10000000000",
///     "total_deposit": "143500600000",
///     "total_airdrop_count": 5,
///     "airdrop_update_candidates": [0, 1, 2, 3],
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
    pub total_airdrop_count: u64,
    pub airdrop_update_candidates: Vec<u64>,
}

/// ## Description
/// Response struct of [`QueryMsg::Claim`] query message
///
/// **Example**
/// ```typescript
/// {
///     "start": 86400,
///     "time": 172800,
///     "amount": "143500600000",
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ClaimResponse {
    pub start: u64,
    pub time: u64,
    pub amount: Uint128,
}

/// ## Description
/// Response struct of [`QueryMsg::Claims`] query message
///
/// **Example**
/// ```typescript
/// {
///     "claims": [
///         [0, { "start": 86400, "time": 172800, "amount": "10000000" }],
///         [1, { "start": 87400, "time": 173800, "amount": "20000000" }],
///         [2, { "start": 88400, "time": 174800, "amount": "30000000" }],
///         ...
///     ],
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ClaimsResponse {
    pub claims: Vec<(u64, ClaimResponse)>,
}

/// ## Description
/// Response struct of [`QueryMsg::Staker`] query message
///
/// **Example**
/// ```typescript
/// {
///     "balance": "1000000",
///     "share": "985000",
///     "latest_claim_id": 10,
///     "last_unlocked_claim_id": 8,
///     "claimable_airdrop": [
///         [ 0, { "token": "terra1xu...kcvhgp", "amount": "100000000" }],
///         ...
///     ],
///     "locked_balance": [
///         [ 0, { "vote": "yes", "balance": "10000" } ],
///         [ 1, { "vote": "no", "balance": "20000" } ],
///         ...
///     ],
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct StakerResponse {
    pub balance: Uint128,
    pub share: Uint128,
    pub latest_claim_id: u64,
    pub last_unlocked_claim_id: Option<u64>,
    pub claimable_airdrop: Vec<(u64, ClaimableAirdrop)>,
    pub locked_balance: Vec<(u64, VoterInfo)>,
}

/// ## Description
/// Response struct of [`QueryMsg::Stakers`] query message
///
/// **Example**
/// ```typescript
/// {
///     "stakers": [
///         [
///             "terra1x...kcgp",
///             {
///                 "balance": "1000000",
///                 "share": "985000",
///                 "latest_claim_id": 10,
///                 "last_unlocked_claim_id": 8,
///                 "claimable_airdrop": [
///                     [ 0, { "token": "terra1x...kcgp", "amount": "100000" }],
///                     ...
///                 ],
///                 "locked_balance": [
///                     [ 0, { "vote": "yes", "balance": "10000" } ],
///                     [ 1, { "vote": "no", "balance": "20000" } ],
///                     ...
///                 ],
///             }
///         ],
///         ...
///     ],
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct StakersResponse {
    pub stakers: Vec<(String, StakerResponse)>,
}

/// ## Description
/// Response struct of [`QueryMsg::Airdrop`] query message
///
/// **Example**
/// ```typescript
/// {
///     "start": 1637672400,
///     "period": 23466600,
///     "reward_token": "terra1x...kcgp",
///     "reward_rate": "951293.759512924752627138"
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct AirdropResponse {
    pub start: u64,
    pub period: u64,
    pub reward_token: String,
    pub reward_rate: Decimal,
}

/// ## Description
/// Response struct of [`QueryMsg::Airdrops`] query message
///
/// **Example**
/// ```typescript
/// {
///     "airdrops": [
///         [
///             3,
///             {
///                 "start": 1644019200,
///                 "period": 31536000,
///                 "reward_token": "terra1x...kcgp",
///                 "reward_rate": "317097.919837645865043125",
///             },
///         ],
///         [
///             2,
///             {
///                 "start": 1637672400,
///                 "period": 27514800,
///                 "reward_token": "terra1x...kcgp",
///                 "reward_rate": "31709.791983732391294866",
///             },
///         ],
///     ],
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct AirdropsResponse {
    pub airdrops: Vec<(u64, AirdropResponse)>,
}

/// ## Description
/// Response struct of [`QueryMsg::Poll`] query message
///
/// **Example**
/// ```typescript
/// {
///     "id": 1,
///     "creator": "terra1x...kcgp",
///     "status": "passed",
///     "end_height": 5325285,
///     "title": "[core] Enabling Decentralized Pylon Governance",
///     "category": "none",
///     "description": "{description}",
///     "link": null,
///     "deposit_amount": "100000000000",
///     "execute_data": null,
///     "yes_votes": "85370155294983",
///     "no_votes": "114964433000",
///     "staked_amount": "302809127264724",
///     "total_balance_at_end_poll": "302809127264724"
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct PollResponse {
    pub id: u64,
    pub creator: String,
    pub status: PollStatus,
    pub end_height: u64,
    pub title: String,
    pub category: PollCategory,
    pub description: String,
    pub link: Option<String>,
    pub deposit_amount: Uint128,
    pub execute_data: Option<Vec<PollExecuteMsg>>,
    pub yes_votes: Uint128, // balance
    pub no_votes: Uint128,  // balance
    pub staked_amount: Option<Uint128>,
    pub total_balance_at_end_poll: Option<Uint128>,
}

/// ## Description
/// Response struct of [`QueryMsg::Polls`] query message
///
/// **Example**
/// ```typescript
/// {
///     "polls": [
///         {
///             "id": 18,
///             "creator": "terra1x...kcgp",
///             "status": "passed",
///             "end_height": 6565540,
///             "title": "The WaterDAO",
///             "category": "gateway",
///             "description": "{description}",
///             "link": null,
///             "deposit_amount": "10000000000",
///             "execute_data": null,
///             "yes_votes": "49021910669996",
///             "no_votes": "1236335908000",
///             "staked_amount": "356178333976671",
///             "total_balance_at_end_poll": "356178333976671"
///         },
///         {
///             "id": 17,
///             "creator": "terra1x...kcgp",
///             "status": "rejected",
///             "end_height": 6452894,
///             "title": "Lunaverse",
///             "category": "gateway",
///             "description": "{description}",
///             "link": null,
///             "deposit_amount": "10000000000",
///             "execute_data": null,
///             "yes_votes": "9149647050995",
///             "no_votes": "28345388098000",
///             "staked_amount": "347230730992651",
///             "total_balance_at_end_poll": "347230730992651"
///         }
///     ]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct PollsResponse {
    pub polls: Vec<PollResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct VotersResponseItem {
    pub voter: String,
    pub vote: VoteOption,
    pub balance: Uint128,
}

/// ## Description
/// Response struct of [`QueryMsg::Voters`] query message
///
/// **Example**
/// ```typescript
/// {
///     "voters": [
///         { "voter": "terra1x...kcgp", "vote": "yes", "balance": "10000" },
///         { "voter": "terra1x...kcgp", "vote": "no",  "balance": "20000" },
///         { "voter": "terra1x...kcgp", "vote": "yes", "balance": "40000" },
///         { "voter": "terra1x...kcgp", "vote": "no",  "balance": "50000" },
///         { "voter": "terra1x...kcgp", "vote": "yes", "balance": "30000" },
///     ]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct VotersResponse {
    pub voters: Vec<VotersResponseItem>,
}
