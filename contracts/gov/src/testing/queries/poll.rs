use crate::queries::poll::{
    query_polls, query_polls_with_category_filter, query_polls_with_status_filter,
};
use cosmwasm_std::{from_binary, Api};
use pylon_token::gov_resp;
use pylon_token::gov_resp::PollsResponse;

use crate::states::poll::{Poll, PollCategory, PollStatus};
use crate::states::state::State;
use crate::testing::instantiate;
use crate::testing::{mock_deps, MockDeps, TEST_CREATOR};

fn save_poll(deps: &mut MockDeps, status: &PollStatus, category: &PollCategory) {
    let api = deps.api;
    let mut state = State::load(deps.as_ref().storage).unwrap();
    let id = state.poll_count;

    Poll::save(
        deps.as_mut().storage,
        &id,
        &Poll {
            id,
            creator: api.addr_canonicalize(TEST_CREATOR).unwrap(),
            status: status.clone(),
            yes_votes: Default::default(),
            no_votes: Default::default(),
            end_height: 0,
            title: "MOCK-POLL".to_string(),
            category: category.clone(),
            description: "MOCK-DESC".to_string(),
            link: None,
            execute_data: None,
            deposit_amount: Default::default(),
            total_balance_at_end_poll: None,
            staked_amount: None,
        },
    )
    .unwrap();

    Poll::index_status(deps.as_mut().storage, &id, status).unwrap();
    Poll::index_category(deps.as_mut().storage, &id, category).unwrap();

    state.poll_count += 1;
    State::save(deps.as_mut().storage, &state).unwrap();
}

fn setup_state(deps: &mut MockDeps) -> (Vec<PollStatus>, Vec<PollCategory>) {
    let status_list = vec![
        PollStatus::InProgress,
        PollStatus::Passed,
        PollStatus::Rejected,
        PollStatus::Executed,
        PollStatus::Failed,
    ];
    let category_list = vec![
        PollCategory::Core,
        PollCategory::Gateway,
        PollCategory::None,
    ];
    for status in status_list.iter() {
        for category in category_list.iter() {
            save_poll(deps, status, category)
        }
    }

    (status_list, category_list)
}

#[test]
fn polls_default() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let (status_list, category_list) = setup_state(&mut deps);

    let response = query_polls(deps.as_ref(), None, None, None).unwrap();
    let response = from_binary::<gov_resp::PollsResponse>(&response).unwrap();
    for status in status_list.iter() {
        for category in category_list.iter() {
            assert!(response
                .polls
                .iter()
                .find(|x| {
                    x.status == status.clone().into() && x.category == category.clone().into()
                })
                .is_some());
        }
    }
}

#[test]
fn polls_with_status_filter_default() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let (_, category_list) = setup_state(&mut deps);

    let response = query_polls_with_status_filter(deps.as_ref(), None, None, None, None).unwrap();
    let response = from_binary::<PollsResponse>(&response).unwrap();
    for category in category_list.iter() {
        assert!(response
            .polls
            .iter()
            .find(|x| x.status == PollStatus::InProgress.into()
                && x.category == category.clone().into())
            .is_some());
        assert!(response
            .polls
            .iter()
            .find(|x| x.status != PollStatus::InProgress.into()
                && x.category == category.clone().into())
            .is_none());
    }
}

#[test]
fn polls_with_category_filter_default() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let (status_list, _) = setup_state(&mut deps);

    let response = query_polls_with_category_filter(deps.as_ref(), None, None, None, None).unwrap();
    let response = from_binary::<PollsResponse>(&response).unwrap();
    for status in status_list.iter() {
        assert!(response
            .polls
            .iter()
            .find(|x| x.status == status.clone().into() && x.category == PollCategory::None.into())
            .is_some());
        assert!(response
            .polls
            .iter()
            .find(|x| x.status == status.clone().into() && x.category != PollCategory::None.into())
            .is_none());
    }
}
