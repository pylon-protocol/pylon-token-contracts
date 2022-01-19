use crate::contract::{execute, instantiate, query_staker_info};
use crate::mock_querier::mock_dependencies;
use crate::state::{read_config, read_state};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{to_binary, Uint128};
use cw20::Cw20ReceiveMsg;
use pylon_token::staking::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};

#[test]
fn migration_test() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        governance: "gov0000".to_string(),
        pylon_token: "reward0000".to_string(),
        staking_token: "staking0000".to_string(),
        distribution_schedule: vec![
            (3585500, 8491943, Uint128::from(750000000000000u128)),
            (8491943, 13398386, Uint128::from(250000000000000u128)),
            (13398386, 18304829, Uint128::from(250000000000000u128)),
            (18304829, 23211272, Uint128::from(250000000000000u128)),
        ],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let mut env = mock_env();
    env.block.height = 3585500u64;
    execute(
        deps.as_mut(),
        env,
        mock_info("staking0000", &[]),
        ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "addr0000".to_string(),
            amount: Uint128::from(1000u128),
            msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
        }),
    )
    .unwrap();

    let msg = ExecuteMsg::MigrateStaking {
        new_staking_contract: "staking0001".to_string(),
    };
    let info = mock_info("gov0000", &[]);
    let mut env = mock_env();
    env.block.height = 6135105u64;

    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    println!("{:?}", read_config(deps.as_ref().storage).unwrap());
    println!("{:?}", read_state(deps.as_ref().storage, 0).unwrap());
    println!("{:?}", read_state(deps.as_ref().storage, 1).unwrap());
    println!(
        "{:?}",
        query_staker_info(deps.as_ref(), "addr0000".to_string(), Some(6135105u64))
    );

    let mut env = mock_env();
    env.block.height = 6135105u64;
    let resp = execute(
        deps.as_mut(),
        env,
        mock_info("staking0001", &[]),
        ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "addr0000".to_string(),
            amount: Uint128::from(1000u128),
            msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
        }),
    )
    .unwrap();
    println!("{:?}", resp);

    let resp = query_staker_info(deps.as_ref(), "addr0000".to_string(), Some(8491943u64)).unwrap();
    println!("{:?}", resp);
}
