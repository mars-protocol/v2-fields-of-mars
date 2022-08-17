use cosmwasm_std::{Addr, Empty, Uint128};
use cw_storage_plus::{Item, Map};

use rover::adapters::{Oracle, RedBank};
use rover::{Denom, NftTokenId, Shares};

// Contract config
pub const OWNER: Item<Addr> = Item::new("owner");
pub const ACCOUNT_NFT: Item<Addr> = Item::new("account_nft");
pub const ALLOWED_COINS: Map<Denom, Empty> = Map::new("allowed_coins");
pub const ALLOWED_VAULTS: Map<&Addr, Empty> = Map::new("allowed_vaults");
pub const RED_BANK: Item<RedBank> = Item::new("red_bank");
pub const ORACLE: Item<Oracle> = Item::new("oracle");

// Positions
pub const COIN_BALANCES: Map<(NftTokenId, Denom), Uint128> = Map::new("coin_balance");
pub const DEBT_SHARES: Map<(NftTokenId, Denom), Shares> = Map::new("debt_shares");
pub const TOTAL_DEBT_SHARES: Map<Denom, Shares> = Map::new("total_debt_shares");