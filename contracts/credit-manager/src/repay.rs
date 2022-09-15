use std::cmp::min;

use cosmwasm_std::{Coin, Deps, DepsMut, Env, Response, Uint128};

use rover::error::{ContractError, ContractResult};
use rover::Shares;

use crate::state::{DEBT_SHARES, RED_BANK, TOTAL_DEBT_SHARES};
use crate::utils::{assert_coin_is_whitelisted, debt_shares_to_amount, decrement_coin_balance};

pub fn repay(deps: DepsMut, env: Env, token_id: &str, coin: Coin) -> ContractResult<Response> {
    if coin.amount.is_zero() {
        return Err(ContractError::NoAmount);
    }

    assert_coin_is_whitelisted(deps.storage, &coin.denom)?;

    // Ensure repayment does not exceed max debt on account
    let (debt_amount, debt_shares) = current_debt_for_denom(deps.as_ref(), &env, token_id, &coin)?;
    let amount_to_repay = min(debt_amount, coin.amount);
    let shares_to_repay = debt_amount_to_shares(
        deps.as_ref(),
        &env,
        &Coin {
            denom: coin.denom.clone(),
            amount: amount_to_repay,
        },
    )?;

    // Decrement token's debt position
    if amount_to_repay == debt_amount {
        DEBT_SHARES.remove(deps.storage, (token_id, &coin.denom));
    } else {
        DEBT_SHARES.save(
            deps.storage,
            (token_id, &coin.denom),
            &debt_shares.checked_sub(shares_to_repay)?,
        )?;
    }

    // Decrement total debt shares for coin
    let total_debt_shares = TOTAL_DEBT_SHARES.load(deps.storage, &coin.denom)?;
    TOTAL_DEBT_SHARES.save(
        deps.storage,
        &coin.denom,
        &total_debt_shares.checked_sub(shares_to_repay)?,
    )?;

    decrement_coin_balance(
        deps.storage,
        token_id,
        &Coin {
            denom: coin.denom.clone(),
            amount: amount_to_repay,
        },
    )?;

    let red_bank = RED_BANK.load(deps.storage)?;
    let red_bank_repay_msg = red_bank.repay_msg(&Coin {
        denom: coin.denom,
        amount: amount_to_repay,
    })?;

    Ok(Response::new()
        .add_message(red_bank_repay_msg)
        .add_attribute("action", "rover/credit_manager/repay")
        .add_attribute("debt_shares_repaid", shares_to_repay)
        .add_attribute("coins_repaid", amount_to_repay))
}

fn debt_amount_to_shares(deps: Deps, env: &Env, coin: &Coin) -> ContractResult<Shares> {
    let red_bank = RED_BANK.load(deps.storage)?;
    let total_debt_shares = TOTAL_DEBT_SHARES.load(deps.storage, &coin.denom)?;
    let total_debt_amount =
        red_bank.query_debt(&deps.querier, &env.contract.address, &coin.denom)?;
    let shares = total_debt_shares.checked_multiply_ratio(coin.amount, total_debt_amount)?;
    Ok(shares)
}

/// Get token's current total debt for denom
pub fn current_debt_for_denom(
    deps: Deps,
    env: &Env,
    token_id: &str,
    coin: &Coin,
) -> ContractResult<(Uint128, Shares)> {
    let debt_shares = DEBT_SHARES
        .load(deps.storage, (token_id, &coin.denom))
        .map_err(|_| ContractError::NoDebt)?;
    let coin = debt_shares_to_amount(deps, &env.contract.address, &coin.denom, debt_shares)?;
    Ok((coin.amount, debt_shares))
}