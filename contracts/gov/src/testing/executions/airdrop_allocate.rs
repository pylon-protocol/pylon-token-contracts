use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, Api, Env, MessageInfo, Uint128};

use crate::error::ContractError;
use crate::executions::airdrop::allocate;
use crate::executions::ExecuteResult;
use crate::states::airdrop::Reward;
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_CREATOR, TEST_TOKEN, TEST_VOTER};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    airdrop_id: u64,
    recipient: String,
    allocate_amount: Uint128,
) -> ExecuteResult {
    allocate(
        deps.as_mut(),
        env,
        info,
        airdrop_id,
        recipient,
        allocate_amount,
    )
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    super::airdrop_instantiate::default(&mut deps, TEST_TOKEN, 1000);

    let response = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_CREATOR, &[]),
        0,
        TEST_VOTER.to_string(),
        Uint128::from(1234u128),
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "airdrop_allocate"),
            attr("airdrop_id", 0.to_string()),
            attr("recipient", TEST_VOTER),
            attr("amount", Uint128::from(1234u128))
        ]
    );

    let reward = Reward::load(
        &deps.storage,
        &deps.api.addr_validate(TEST_VOTER).unwrap(),
        &0u64,
    )
    .unwrap();
    assert_eq!(reward.reward, Uint128::from(1234u128));
}

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    super::airdrop_instantiate::default(&mut deps, TEST_TOKEN, 1000);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        0,
        TEST_VOTER.to_string(),
        Uint128::from(1234u128),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
