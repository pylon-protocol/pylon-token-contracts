use cosmwasm_std::from_binary;
use pylon_token::gov_resp::APIVersionResponse;

use crate::constant::CONTRACT_VERSION;
use crate::queries::query_api_version;
use crate::testing::instantiate;
use crate::testing::mock_deps;

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    let response = query_api_version(deps.as_ref()).unwrap();
    let response: APIVersionResponse = from_binary(&response).unwrap();
    assert_eq!(response.version, CONTRACT_VERSION.to_string());
}
