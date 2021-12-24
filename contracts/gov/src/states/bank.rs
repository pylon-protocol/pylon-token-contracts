use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use pylon_token::common::OrderBy;
use pylon_utils::range::{
    calc_range_end, calc_range_end_addr, calc_range_start, calc_range_start_addr,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

use crate::constant::{DEFAULT_QUERY_LIMIT, MAX_QUERY_LIMIT};
use crate::states::poll::VoterInfo;

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenClaim {
    pub time: u64,
    pub amount: Uint128,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyTokenManager {
    pub share: Uint128, // total staked balance
    pub latest_claim_id: Option<u64>,
    pub last_unlocked_claim_id: Option<u64>,
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenManager {
    pub share: Uint128, // total staked balance
    pub latest_claim_id: u64,
    pub last_unlocked_claim_id: u64,
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}

impl TokenManager {
    pub fn load(storage: &dyn Storage, address: &CanonicalAddr) -> StdResult<TokenManager> {
        let token_manager: LegacyTokenManager = ReadonlyBucket::new(storage, super::PREFIX_BANK)
            .load(address.as_slice())
            .unwrap_or_default();

        Ok(TokenManager {
            share: token_manager.share,
            latest_claim_id: token_manager.latest_claim_id.unwrap_or_default(),
            last_unlocked_claim_id: token_manager.last_unlocked_claim_id.unwrap_or_default(),
            locked_balance: token_manager.locked_balance,
        })
    }

    pub fn load_range(
        storage: &dyn Storage,
        start_after: Option<CanonicalAddr>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<(CanonicalAddr, TokenManager)>> {
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
        };
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

        ReadonlyBucket::new(storage, super::PREFIX_BANK)
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(
                |elem: StdResult<(Vec<u8>, LegacyTokenManager)>| -> StdResult<(CanonicalAddr, TokenManager)> {
                    let (k, v) = elem.unwrap();

                    Ok((CanonicalAddr::from(k), TokenManager{
                        share: v.share,
                        latest_claim_id: v.latest_claim_id.unwrap_or_default(),
                        last_unlocked_claim_id: v.last_unlocked_claim_id.unwrap_or_default(),
                        locked_balance: v.locked_balance
                    }))
                },
            )
            .collect()
    }

    pub fn load_claim(
        storage: &dyn Storage,
        address: &CanonicalAddr,
        claim_id: u64,
    ) -> StdResult<TokenClaim> {
        ReadonlyBucket::multilevel(
            storage,
            &[super::PREFIX_BANK_UNLOCK_CLAIM, address.as_slice()],
        )
        .load(&claim_id.to_be_bytes())
    }

    pub fn load_claim_range(
        storage: &dyn Storage,
        address: &CanonicalAddr,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<(u64, TokenClaim)>> {
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end(start_after), OrderBy::Desc),
        };
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

        ReadonlyBucket::multilevel(
            storage,
            &[super::PREFIX_BANK_UNLOCK_CLAIM, address.as_slice()],
        )
        .range(start.as_deref(), end.as_deref(), order_by.into())
        .take(limit)
        .map(
            |elem: StdResult<(Vec<u8>, TokenClaim)>| -> StdResult<(u64, TokenClaim)> {
                let (k, v) = elem.unwrap();
                Ok((u64::from_be_bytes(k.try_into().unwrap()), v))
            },
        )
        .collect()
    }

    pub fn save(
        storage: &mut dyn Storage,
        address: &CanonicalAddr,
        manager: &TokenManager,
    ) -> StdResult<()> {
        Bucket::new(storage, super::PREFIX_BANK).save(address.as_slice(), manager)
    }

    pub fn save_claim(
        storage: &mut dyn Storage,
        address: &CanonicalAddr,
        id: u64,
        claim: TokenClaim,
    ) -> StdResult<()> {
        Bucket::multilevel(
            storage,
            &[super::PREFIX_BANK_UNLOCK_CLAIM, address.as_slice()],
        )
        .save(&id.to_be_bytes(), &claim)
    }
}
