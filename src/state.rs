use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ Addr, Uint128 };
use cw_storage_plus::{ Item, Map };

#[cw_serde]
pub struct State {
    pub admin: Addr,
    pub token_info: TokenInfo,
    pub buy_denom: String,
    pub buy_token_type: String,
    pub tokens_per_buy: Uint128,
    pub start_time: u64,
    pub end_time: u64,
    pub phases: Vec<Phase>,
    pub claimable: bool,
    pub fees_wallet: Addr,
    pub fees_percentage: u64,
    pub withdraw_wallet: Addr,
    pub whitelist: Vec<WhitelistUser>,
}

#[cw_serde]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub denom: Option<String>,
    pub decimals: Option<u8>,
    pub description: String,
    pub supply: u64,
}
#[cw_serde]
pub struct User {
    pub buy_phases: HashMap<String, u64>,
    pub amount: u64,
    pub tokens: Uint128,
    pub claimed: bool,
}

#[cw_serde]
pub struct WhitelistUser {
    pub address: Addr,
    pub amount: u64,
}

#[cw_serde]
pub struct Phase {
    pub name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub price_per_token: Uint128,
    pub supply: u64,
    pub address_list: Vec<Addr>,
    pub limit: u64,
    pub total_minted: u64,
}

#[cw_serde]
pub struct PhaseResp {
    pub name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub price_per_token: Uint128,
    pub supply: u64,
    pub address_list: u64,
    pub limit: u64,
    pub total_minted: u64,
}

#[cw_serde]
pub struct PhaseInformation {
    pub limit: u64,
    pub current_mint: u64,
    pub eligible: bool,
}

#[cw_serde]
pub struct UserToken {
    pub address: Addr,
    pub tokens: Uint128,
}

pub const STATE_KEY: &str = "state";
pub const STATE: Item<State> = Item::new(STATE_KEY);
pub const USER_INFO: Map<Addr, User> = Map::new("users");
pub const USERS: Map<String, Vec<UserToken>> = Map::new("user_tokens");
