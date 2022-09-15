use cosmwasm_std::OverflowOperation::Sub;
use cosmwasm_std::{coin, Addr, Coin, Decimal, OverflowError, Uint128};

use rover::error::ContractError;
use rover::msg::execute::Action::{Deposit, SwapExactIn};
use swapper_mock::contract::MOCK_SWAP_RESULT;

use crate::helpers::{assert_err, uatom_info, uosmo_info, AccountToFund, MockEnv};

pub mod helpers;

#[test]
fn test_only_token_owner_can_swap_for_account() {
    let user = Addr::unchecked("user");
    let mut mock = MockEnv::new().build().unwrap();
    let token_id = mock.create_credit_account(&user).unwrap();

    let another_user = Addr::unchecked("another_user");
    let res = mock.update_credit_account(
        &token_id,
        &another_user,
        vec![SwapExactIn {
            coin_in: coin(12, "mars"),
            denom_out: "osmo".to_string(),
            slippage: Decimal::from_atomics(6u128, 1).unwrap(),
        }],
        &[],
    );

    assert_err(
        res,
        ContractError::NotTokenOwner {
            user: another_user.into(),
            token_id,
        },
    )
}

#[test]
fn test_coin_in_must_be_whitelisted() {
    let user = Addr::unchecked("user");
    let mut mock = MockEnv::new().build().unwrap();
    let token_id = mock.create_credit_account(&user).unwrap();

    let res = mock.update_credit_account(
        &token_id,
        &user,
        vec![SwapExactIn {
            coin_in: coin(12, "mars"),
            denom_out: "osmo".to_string(),
            slippage: Decimal::from_atomics(6u128, 1).unwrap(),
        }],
        &[],
    );

    assert_err(res, ContractError::NotWhitelisted("mars".to_string()))
}

#[test]
fn test_denom_out_must_be_whitelisted() {
    let osmo_info = uosmo_info();

    let user = Addr::unchecked("user");
    let mut mock = MockEnv::new()
        .allowed_coins(&[osmo_info.clone()])
        .build()
        .unwrap();
    let token_id = mock.create_credit_account(&user).unwrap();

    let res = mock.update_credit_account(
        &token_id,
        &user,
        vec![SwapExactIn {
            coin_in: osmo_info.to_coin(Uint128::new(10_000)),
            denom_out: "ujake".to_string(),
            slippage: Decimal::from_atomics(6u128, 1).unwrap(),
        }],
        &[],
    );

    assert_err(res, ContractError::NotWhitelisted("ujake".to_string()))
}

#[test]
fn test_no_amount_sent() {
    let osmo_info = uosmo_info();
    let atom_info = uatom_info();

    let user = Addr::unchecked("user");
    let mut mock = MockEnv::new()
        .allowed_coins(&[osmo_info.clone(), atom_info.clone()])
        .build()
        .unwrap();
    let token_id = mock.create_credit_account(&user).unwrap();

    let res = mock.update_credit_account(
        &token_id,
        &user,
        vec![SwapExactIn {
            coin_in: osmo_info.to_coin(Uint128::zero()),
            denom_out: atom_info.denom,
            slippage: Decimal::from_atomics(6u128, 1).unwrap(),
        }],
        &[],
    );

    assert_err(res, ContractError::NoAmount)
}

#[test]
fn test_user_has_zero_balance_for_swap_req() {
    let osmo_info = uosmo_info();
    let atom_info = uatom_info();

    let user = Addr::unchecked("user");
    let mut mock = MockEnv::new()
        .allowed_coins(&[osmo_info.clone(), atom_info.clone()])
        .build()
        .unwrap();
    let token_id = mock.create_credit_account(&user).unwrap();

    let res = mock.update_credit_account(
        &token_id,
        &user,
        vec![SwapExactIn {
            coin_in: osmo_info.to_coin(Uint128::new(10_000)),
            denom_out: atom_info.denom,
            slippage: Decimal::from_atomics(6u128, 1).unwrap(),
        }],
        &[],
    );

    assert_err(
        res,
        ContractError::Overflow(OverflowError {
            operation: Sub,
            operand1: "0".to_string(),
            operand2: "10000".to_string(),
        }),
    )
}

#[test]
fn test_user_does_not_have_enough_balance_for_swap_req() {
    let osmo_info = uosmo_info();
    let atom_info = uatom_info();

    let user = Addr::unchecked("user");
    let mut mock = MockEnv::new()
        .allowed_coins(&[osmo_info.clone(), atom_info.clone()])
        .fund_account(AccountToFund {
            addr: user.clone(),
            funds: vec![Coin::new(300u128, osmo_info.denom.clone())],
        })
        .build()
        .unwrap();
    let token_id = mock.create_credit_account(&user).unwrap();

    let res = mock.update_credit_account(
        &token_id,
        &user,
        vec![
            Deposit(osmo_info.to_coin(Uint128::new(100))),
            SwapExactIn {
                coin_in: osmo_info.to_coin(Uint128::new(10_000)),
                denom_out: atom_info.denom,
                slippage: Decimal::from_atomics(6u128, 1).unwrap(),
            },
        ],
        &[osmo_info.to_coin(Uint128::new(100))],
    );

    assert_err(
        res,
        ContractError::Overflow(OverflowError {
            operation: Sub,
            operand1: "100".to_string(),
            operand2: "10000".to_string(),
        }),
    )
}

#[test]
fn test_swap_successful() {
    let atom_info = uatom_info();
    let osmo_info = uosmo_info();

    let user = Addr::unchecked("user");
    let mut mock = MockEnv::new()
        .allowed_coins(&[osmo_info.clone(), atom_info.clone()])
        .fund_account(AccountToFund {
            addr: user.clone(),
            funds: vec![Coin::new(10_000u128, atom_info.denom.clone())],
        })
        .build()
        .unwrap();

    let res = mock.query_swap_estimate(&atom_info.to_coin(Uint128::new(10_000)), &osmo_info.denom);
    assert_eq!(res.amount, MOCK_SWAP_RESULT);

    let token_id = mock.create_credit_account(&user).unwrap();
    mock.update_credit_account(
        &token_id,
        &user,
        vec![
            Deposit(atom_info.to_coin(Uint128::new(10_000))),
            SwapExactIn {
                coin_in: atom_info.to_coin(Uint128::new(10_000)),
                denom_out: osmo_info.denom.clone(),
                slippage: Decimal::from_atomics(6u128, 1).unwrap(),
            },
        ],
        &[atom_info.to_coin(Uint128::new(10_000))],
    )
    .unwrap();

    // assert rover balance
    let atom_balance = mock.query_balance(&mock.rover, &atom_info.denom).amount;
    let osmo_balance = mock.query_balance(&mock.rover, &osmo_info.denom).amount;
    assert_eq!(atom_balance, Uint128::zero());
    assert_eq!(osmo_balance, MOCK_SWAP_RESULT);

    // assert account position
    let position = mock.query_position(&token_id);
    assert_eq!(position.coins.len(), 1);
    assert_eq!(position.coins.first().unwrap().denom, osmo_info.denom);
    assert_eq!(position.coins.first().unwrap().amount, MOCK_SWAP_RESULT);
}