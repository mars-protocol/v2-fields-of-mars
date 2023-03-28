use cosmwasm_std::{Addr, Coin, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;
use mars_rover::{
    adapters::vault::{Vault, VaultBase, VaultPosition, VaultUnchecked},
    error::ContractResult,
    msg::query::{
        CoinBalanceResponseItem, ConfigResponse, DebtAmount, DebtShares, Positions,
        SharesResponseItem, VaultInfoResponse, VaultPositionResponseItem, VaultWithBalance,
    },
};

use crate::{
    state::{
        ACCOUNT_NFT, ALLOWED_COINS, COIN_BALANCES, DEBT_SHARES, MAX_CLOSE_FACTOR,
        MAX_UNLOCKING_POSITIONS, ORACLE, OWNER, RED_BANK, SWAPPER, TOTAL_DEBT_SHARES,
        VAULT_CONFIGS, VAULT_POSITIONS, ZAPPER,
    },
    utils::debt_shares_to_amount,
    vault::vault_utilization_in_deposit_cap_denom,
};

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_config(deps: Deps) -> ContractResult<ConfigResponse> {
    Ok(ConfigResponse {
        ownership: OWNER.query(deps.storage)?,
        account_nft: ACCOUNT_NFT.may_load(deps.storage)?.map(|addr| addr.to_string()),
        red_bank: RED_BANK.load(deps.storage)?.address().into(),
        oracle: ORACLE.load(deps.storage)?.address().into(),
        max_close_factor: MAX_CLOSE_FACTOR.load(deps.storage)?,
        max_unlocking_positions: MAX_UNLOCKING_POSITIONS.load(deps.storage)?,
        swapper: SWAPPER.load(deps.storage)?.address().into(),
        zapper: ZAPPER.load(deps.storage)?.address().into(),
    })
}

pub fn query_positions(deps: Deps, env: &Env, account_id: &str) -> ContractResult<Positions> {
    Ok(Positions {
        account_id: account_id.to_string(),
        deposits: query_coin_balances(deps, account_id)?,
        debts: query_debt_amounts(deps, env, account_id)?,
        vaults: query_vault_positions(deps, account_id)?,
    })
}

pub fn query_all_coin_balances(
    deps: Deps,
    start_after: Option<(String, String)>,
    limit: Option<u32>,
) -> StdResult<Vec<CoinBalanceResponseItem>> {
    let start = start_after
        .as_ref()
        .map(|(account_id, denom)| Bound::exclusive((account_id.as_str(), denom.as_str())));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    Ok(COIN_BALANCES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?
        .iter()
        .map(|((account_id, denom), amount)| CoinBalanceResponseItem {
            account_id: account_id.to_string(),
            denom: denom.to_string(),
            amount: *amount,
        })
        .collect())
}

fn query_debt_amounts(deps: Deps, env: &Env, account_id: &str) -> ContractResult<Vec<DebtAmount>> {
    DEBT_SHARES
        .prefix(account_id)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|res| {
            let (denom, shares) = res?;
            let coin = debt_shares_to_amount(deps, &env.contract.address, &denom, shares)?;
            Ok(DebtAmount {
                denom,
                shares,
                amount: coin.amount,
            })
        })
        .collect()
}

pub fn query_coin_balances(deps: Deps, account_id: &str) -> ContractResult<Vec<Coin>> {
    COIN_BALANCES
        .prefix(account_id)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (denom, amount) = item?;
            Ok(Coin {
                denom,
                amount,
            })
        })
        .collect()
}

pub fn query_all_debt_shares(
    deps: Deps,
    start_after: Option<(String, String)>,
    limit: Option<u32>,
) -> StdResult<Vec<SharesResponseItem>> {
    let start = start_after
        .as_ref()
        .map(|(account_id, denom)| Bound::exclusive((account_id.as_str(), denom.as_str())));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    Ok(DEBT_SHARES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?
        .iter()
        .map(|((account_id, denom), shares)| SharesResponseItem {
            account_id: account_id.to_string(),
            denom: denom.to_string(),
            shares: *shares,
        })
        .collect())
}

pub fn query_vaults_info(
    deps: Deps,
    env: Env,
    start_after: Option<VaultUnchecked>,
    limit: Option<u32>,
) -> ContractResult<Vec<VaultInfoResponse>> {
    let vault: Vault;
    let start = match &start_after {
        Some(unchecked) => {
            vault = unchecked.check(deps.api)?;
            Some(Bound::exclusive(&vault.address))
        }
        None => None,
    };

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    VAULT_CONFIGS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| {
            let (addr, config) = res?;
            let vault = VaultBase::new(addr);
            Ok(VaultInfoResponse {
                vault: vault.clone().into(),
                config,
                utilization: vault_utilization_in_deposit_cap_denom(
                    &deps,
                    &vault,
                    &env.contract.address,
                )?,
            })
        })
        .collect()
}

pub fn query_vault_positions(deps: Deps, account_id: &str) -> ContractResult<Vec<VaultPosition>> {
    VAULT_POSITIONS
        .prefix(account_id)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|res| {
            let (addr, position) = res?;
            Ok(VaultPosition {
                vault: VaultBase::new(addr),
                amount: position,
            })
        })
        .collect()
}

pub fn query_all_vault_positions(
    deps: Deps,
    start_after: Option<(String, String)>,
    limit: Option<u32>,
) -> StdResult<Vec<VaultPositionResponseItem>> {
    let start = match &start_after {
        Some((account_id, unchecked)) => {
            let addr = deps.api.addr_validate(unchecked)?;
            Some(Bound::exclusive((account_id.as_str(), addr)))
        }
        None => None,
    };

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    Ok(VAULT_POSITIONS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?
        .iter()
        .map(|((account_id, addr), amount)| VaultPositionResponseItem {
            account_id: account_id.clone(),
            position: VaultPosition {
                vault: VaultBase::new(addr.clone()),
                amount: amount.clone(),
            },
        })
        .collect())
}

/// NOTE: This implementation of the query function assumes the map `ALLOWED_COINS` only saves `Empty`.
/// If a coin is to be removed from the whitelist, the map must remove the corresponding key.
pub fn query_allowed_coins(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<String>> {
    let start = start_after.as_ref().map(|denom| Bound::exclusive(denom.as_str()));

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    ALLOWED_COINS
        .items(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_total_debt_shares(deps: Deps, denom: &str) -> StdResult<DebtShares> {
    let shares = TOTAL_DEBT_SHARES.load(deps.storage, denom)?;
    Ok(DebtShares {
        denom: denom.to_string(),
        shares,
    })
}

pub fn query_all_total_debt_shares(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<DebtShares>> {
    let start = start_after.as_ref().map(|denom| Bound::exclusive(denom.as_str()));

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    Ok(TOTAL_DEBT_SHARES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?
        .iter()
        .map(|(denom, shares)| DebtShares {
            denom: denom.to_string(),
            shares: *shares,
        })
        .collect())
}

pub fn query_total_vault_coin_balance(
    deps: Deps,
    unchecked: &VaultUnchecked,
    rover_addr: &Addr,
) -> StdResult<Uint128> {
    let vault = unchecked.check(deps.api)?;
    vault.query_balance(&deps.querier, rover_addr)
}

pub fn query_all_total_vault_coin_balances(
    deps: Deps,
    rover_addr: &Addr,
    start_after: Option<VaultUnchecked>,
    limit: Option<u32>,
) -> StdResult<Vec<VaultWithBalance>> {
    let vault: Vault;
    let start = match &start_after {
        Some(unchecked) => {
            vault = unchecked.check(deps.api)?;
            Some(Bound::exclusive(&vault.address))
        }
        None => None,
    };

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    VAULT_CONFIGS
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| {
            let addr = res?;
            let vault = VaultBase::new(addr);
            let balance = vault.query_balance(&deps.querier, rover_addr)?;
            Ok(VaultWithBalance {
                vault,
                balance,
            })
        })
        .collect()
}
