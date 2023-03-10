use cosmwasm_std::{Addr, Uint128};
use mars_rover::adapters::account_nft::NftConfigUpdates;

use crate::helpers::MockEnv;

pub mod helpers;

#[test]
fn only_minter_can_update_config() {
    let mut mock = MockEnv::new().build().unwrap();

    let bad_guy = Addr::unchecked("bad_guy");
    let res = mock.update_config(
        &bad_guy,
        &NftConfigUpdates {
            max_value_for_burn: None,
            proposed_new_minter: None,
        },
    );

    if res.is_ok() {
        panic!("Non-minter should not be able to propose new minter");
    }
}

#[test]
fn minter_can_update_config() {
    let mut mock = MockEnv::new().build().unwrap();

    let new_max_burn_val = Uint128::new(4918453);
    let new_proposed_minter = "new_proposed_minter".to_string();

    let updates = NftConfigUpdates {
        max_value_for_burn: Some(new_max_burn_val),
        proposed_new_minter: Some(new_proposed_minter.clone()),
    };

    mock.update_config(&mock.minter.clone(), &updates).unwrap();

    let config = mock.query_config();
    assert_eq!(config.max_value_for_burn, new_max_burn_val);
    assert_eq!(config.proposed_new_minter.unwrap(), new_proposed_minter);
}
