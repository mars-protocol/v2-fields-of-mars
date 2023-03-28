use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response,
    StdError,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use mars_health::HealthResponse;
use mars_rover::{
    adapters::vault::VAULT_REQUEST_REPLY_ID,
    error::{ContractError, ContractError::Migration, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};
use semver::Version;

use crate::{
    emergency_update::emergency_config_update,
    execute::{create_credit_account, dispatch_actions, execute_callback},
    health::compute_health,
    instantiate::store_config,
    query::{
        query_all_coin_balances, query_all_debt_shares, query_all_total_debt_shares,
        query_all_total_vault_coin_balances, query_all_vault_positions, query_allowed_coins,
        query_config, query_positions, query_total_debt_shares, query_total_vault_coin_balance,
        query_vaults_info,
    },
    update_config::{update_config, update_nft_config, update_owner},
    vault::handle_unlock_request_reply,
    zap::{estimate_provide_liquidity, estimate_withdraw_liquidity},
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    set_contract_version(deps.storage, format!("crates.io:{CONTRACT_NAME}"), CONTRACT_VERSION)?;
    store_config(deps, &msg)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::CreateCreditAccount {} => create_credit_account(deps, info.sender),
        ExecuteMsg::UpdateConfig {
            updates,
        } => update_config(deps, info, updates),
        ExecuteMsg::UpdateNftConfig {
            updates,
        } => update_nft_config(deps, info, updates),
        ExecuteMsg::UpdateOwner(update) => update_owner(deps, info, update),
        ExecuteMsg::Callback(callback) => execute_callback(deps, info, env, callback),
        ExecuteMsg::UpdateCreditAccount {
            account_id,
            actions,
        } => dispatch_actions(deps, env, info, &account_id, &actions),
        ExecuteMsg::EmergencyConfigUpdate(update) => emergency_config_update(deps, info, update),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _: Env, reply: Reply) -> ContractResult<Response> {
    match reply.id {
        VAULT_REQUEST_REPLY_ID => handle_unlock_request_reply(deps, reply),
        id => Err(ContractError::ReplyIdError(id)),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let res = match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::VaultsInfo {
            start_after,
            limit,
        } => to_binary(&query_vaults_info(deps, env, start_after, limit)?),
        QueryMsg::AllowedCoins {
            start_after,
            limit,
        } => to_binary(&query_allowed_coins(deps, start_after, limit)?),
        QueryMsg::Positions {
            account_id,
        } => to_binary(&query_positions(deps, &env, &account_id)?),
        QueryMsg::Health {
            account_id,
        } => to_binary::<HealthResponse>(&Into::into(compute_health(deps, &env, &account_id)?)),
        QueryMsg::AllCoinBalances {
            start_after,
            limit,
        } => to_binary(&query_all_coin_balances(deps, start_after, limit)?),
        QueryMsg::AllDebtShares {
            start_after,
            limit,
        } => to_binary(&query_all_debt_shares(deps, start_after, limit)?),
        QueryMsg::TotalDebtShares(denom) => to_binary(&query_total_debt_shares(deps, &denom)?),
        QueryMsg::AllTotalDebtShares {
            start_after,
            limit,
        } => to_binary(&query_all_total_debt_shares(deps, start_after, limit)?),
        QueryMsg::TotalVaultCoinBalance {
            vault,
        } => to_binary(&query_total_vault_coin_balance(deps, &vault, &env.contract.address)?),
        QueryMsg::AllTotalVaultCoinBalances {
            start_after,
            limit,
        } => to_binary(&query_all_total_vault_coin_balances(
            deps,
            &env.contract.address,
            start_after,
            limit,
        )?),
        QueryMsg::AllVaultPositions {
            start_after,
            limit,
        } => to_binary(&query_all_vault_positions(deps, start_after, limit)?),
        QueryMsg::EstimateProvideLiquidity {
            lp_token_out,
            coins_in,
        } => to_binary(&estimate_provide_liquidity(deps, &lp_token_out, coins_in)?),
        QueryMsg::EstimateWithdrawLiquidity {
            lp_token,
        } => to_binary(&estimate_withdraw_liquidity(deps, lp_token)?),
    };
    res.map_err(Into::into)
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> ContractResult<Response> {
    let ContractVersion {
        contract: storage_contract,
        version,
    } = get_contract_version(deps.storage)?;

    if storage_contract != format!("crates.io:{CONTRACT_NAME}") {
        return Err(Migration("Can only upgrade from same type".to_string()));
    }

    let storage_version = version
        .parse::<Version>()
        .map_err(|_| StdError::generic_err("storage version parse err"))?;
    let new_version = CONTRACT_VERSION
        .parse::<Version>()
        .map_err(|_| StdError::generic_err("new version parse err"))?;

    if storage_version >= new_version {
        return Err(Migration("Cannot upgrade from the current or newer version".to_string()));
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("from_version", version)
        .add_attribute("to_version", CONTRACT_VERSION))
}
