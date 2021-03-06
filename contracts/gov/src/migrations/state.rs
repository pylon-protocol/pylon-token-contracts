use crate::constant::{CONTRACT_NAME, CONTRACT_VERSION};
use cosmwasm_std::{DepsMut, Env, Order, Response};
use cosmwasm_storage::ReadonlyBucket;
use cw2::set_contract_version;

use crate::states::poll::Poll;
use crate::states::PREFIX_POLL;

pub fn migrate(deps: DepsMut, _env: Env) -> super::MigrateResult {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();

    let polls: Vec<Poll> = ReadonlyBucket::<Poll>::new(deps.storage, PREFIX_POLL)
        .range(None, None, Order::Ascending)
        .map(|item| -> Poll {
            let (_, v) = item.unwrap();
            v
        })
        .collect();

    for poll in polls.iter() {
        Poll::index_status(deps.storage, &poll.id, &poll.status).unwrap();
        Poll::index_category(deps.storage, &poll.id, &poll.category).unwrap();
    }

    Ok(Response::default())
}
