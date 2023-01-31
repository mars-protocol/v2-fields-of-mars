use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, MessageInfo, Response, WasmMsg};
use mars_owner::OwnerUpdate;
use mars_rover::{
    adapters::account_nft::{ExecuteMsg as NftExecuteMsg, NftConfigUpdates},
    error::ContractResult,
    msg::instantiate::ConfigUpdates,
    traits::{FallbackStr, Stringify},
};

use crate::{
    instantiate::{assert_lte_to_one, assert_no_duplicate_coins, assert_no_duplicate_vaults},
    state::{
        ACCOUNT_NFT, ALLOWED_COINS, MAX_CLOSE_FACTOR, MAX_UNLOCKING_POSITIONS, ORACLE, OWNER,
        SWAPPER, VAULT_CONFIGS, ZAPPER,
    },
};

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    updates: ConfigUpdates,
) -> ContractResult<Response> {
    OWNER.assert_owner(deps.storage, &info.sender)?;

    let mut response = Response::new().add_attribute("action", "update_config");

    if let Some(addr_str) = updates.account_nft {
        let validated = deps.api.addr_validate(&addr_str)?;
        ACCOUNT_NFT.save(deps.storage, &validated)?;

        // Accept minter role. NFT contract minter must have proposed Rover as a new minter first.
        let accept_minter_role_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: addr_str.clone(),
            funds: vec![],
            msg: to_binary(&NftExecuteMsg::AcceptMinterRole {})?,
        });

        response = response
            .add_message(accept_minter_role_msg)
            .add_attribute("key", "account_nft")
            .add_attribute("value", addr_str);
    }

    if let Some(coins) = updates.allowed_coins {
        assert_no_duplicate_coins(&coins)?;
        ALLOWED_COINS.clear(deps.storage);
        coins.iter().try_for_each(|denom| ALLOWED_COINS.insert(deps.storage, denom).map(|_| ()))?;

        response = response
            .add_attribute("key", "allowed_coins")
            .add_attribute("value", coins.join(", ").fallback("None"));
    }

    if let Some(configs) = updates.vault_configs {
        assert_no_duplicate_vaults(deps.api, &deps.querier, &configs)?;
        VAULT_CONFIGS.clear(deps.storage);
        configs.iter().try_for_each(|v| -> ContractResult<_> {
            v.config.check()?;
            let vault = v.vault.check(deps.api)?;
            Ok(VAULT_CONFIGS.save(deps.storage, &vault.address, &v.config)?)
        })?;
        response = response
            .add_attribute("key", "vault_configs")
            .add_attribute("value", configs.to_string().fallback("None"))
    }

    if let Some(unchecked) = updates.oracle {
        ORACLE.save(deps.storage, &unchecked.check(deps.api)?)?;
        response =
            response.add_attribute("key", "oracle").add_attribute("value", unchecked.address());
    }

    if let Some(unchecked) = updates.swapper {
        SWAPPER.save(deps.storage, &unchecked.check(deps.api)?)?;
        response =
            response.add_attribute("key", "swapper").add_attribute("value", unchecked.address());
    }

    if let Some(unchecked) = updates.zapper {
        ZAPPER.save(deps.storage, &unchecked.check(deps.api)?)?;
        response =
            response.add_attribute("key", "zapper").add_attribute("value", unchecked.address());
    }

    if let Some(cf) = updates.max_close_factor {
        assert_lte_to_one(&cf)?;
        MAX_CLOSE_FACTOR.save(deps.storage, &cf)?;
        response = response
            .add_attribute("key", "max_close_factor")
            .add_attribute("value", cf.to_string());
    }

    if let Some(num) = updates.max_unlocking_positions {
        MAX_UNLOCKING_POSITIONS.save(deps.storage, &num)?;
        response = response
            .add_attribute("key", "max_unlocking_positions")
            .add_attribute("value", num.to_string());
    }

    Ok(response)
}

pub fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    update: OwnerUpdate,
) -> ContractResult<Response> {
    Ok(OWNER.update(deps, info, update)?)
}

pub fn update_nft_config(
    deps: DepsMut,
    info: MessageInfo,
    updates: NftConfigUpdates,
) -> ContractResult<Response> {
    OWNER.assert_owner(deps.storage, &info.sender)?;

    let nft_contract = ACCOUNT_NFT.load(deps.storage)?;

    let update_config_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: nft_contract.to_string(),
        funds: vec![],
        msg: to_binary(&NftExecuteMsg::UpdateConfig {
            updates,
        })?,
    });

    Ok(Response::new().add_attribute("action", "update_nft_config").add_message(update_config_msg))
}
