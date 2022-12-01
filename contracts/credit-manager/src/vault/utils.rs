use cosmwasm_std::{Addr, Coin, Deps, StdResult, Storage, Uint128};

use mars_rover::adapters::vault::{Vault, VaultPositionAmount, VaultPositionUpdate};
use mars_rover::error::{ContractError, ContractResult};

use crate::state::{MAX_UNLOCKING_POSITIONS, VAULT_CONFIGS, VAULT_POSITIONS};
use crate::update_coin_balances::query_balance;

pub fn assert_vault_is_whitelisted(storage: &mut dyn Storage, vault: &Vault) -> ContractResult<()> {
    let config = VAULT_CONFIGS
        .may_load(storage, &vault.address)?
        .and_then(|config| config.whitelisted.then_some(true));
    if config.is_none() {
        return Err(ContractError::NotWhitelisted(vault.address.to_string()));
    }
    Ok(())
}

pub fn assert_under_max_unlocking_limit(
    storage: &mut dyn Storage,
    account_id: &str,
    vault: &Vault,
) -> ContractResult<()> {
    let maximum = MAX_UNLOCKING_POSITIONS.load(storage)?;
    let new_amount = VAULT_POSITIONS
        .may_load(storage, (account_id, vault.address.clone()))?
        .map(|p| p.unlocking().positions().len())
        .map(|len| Uint128::from(len as u128))
        .unwrap_or(Uint128::zero())
        .checked_add(Uint128::one())?;

    if new_amount > maximum {
        return Err(ContractError::ExceedsMaxUnlockingPositions {
            new_amount,
            maximum,
        });
    }
    Ok(())
}

pub fn update_vault_position(
    storage: &mut dyn Storage,
    account_id: &str,
    vault_addr: &Addr,
    update: VaultPositionUpdate,
) -> ContractResult<VaultPositionAmount> {
    let path = VAULT_POSITIONS.key((account_id, vault_addr.clone()));
    let mut amount = path
        .may_load(storage)?
        .unwrap_or_else(|| update.default_amount());

    amount.update(update)?;

    if amount.is_empty() {
        path.remove(storage);
    } else {
        path.save(storage, &amount)?;
    }
    Ok(amount)
}

/// Returns the total vault token balance for rover
pub fn query_withdraw_denom_balance(
    deps: Deps,
    rover_addr: &Addr,
    vault: &Vault,
) -> StdResult<Coin> {
    let vault_info = vault.query_info(&deps.querier)?;
    query_balance(&deps.querier, rover_addr, vault_info.base_token.as_str())
}
