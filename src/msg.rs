use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ Addr, Uint128 };

use crate::state::{ Phase, PhaseInformation, PhaseResp, TokenInfo, WhitelistUser };

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub start_time: u64,
    pub end_time: u64,
    pub buy_denom: String,
    pub buy_token_type: String,
    pub tokens_per_buy: Uint128,
    pub token_info: TokenInfo,
    pub phases: Option<Vec<Phase>>,
    pub fees_wallet: Addr,
    pub withdraw_wallet: Addr,
    pub whitelist: Option<Vec<WhitelistUser>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        start_time: Option<u64>,
        end_time: Option<u64>,
        buy_denom: Option<String>,
        buy_token_type: Option<String>,
        tokens_per_buy: Option<Uint128>,
        token_info: Option<TokenInfo>,
        phases: Option<Vec<Phase>>,
        new_admin: Option<Addr>,
    },
    AddPhase {
        phase: Phase,
    },
    UpdatePhase {
        name: String,
        start_time: Option<u64>,
        end_time: Option<u64>,
        price_per_token: Option<Uint128>,
        supply: Option<u64>,
        address_list: Option<Vec<Addr>>,
        limit: Option<u64>,
    },
    RemovePhase {
        name: String,
    },
    AddWhitelist {
        whitelist: Vec<WhitelistUser>,
    },
    RemoveFromWhitelist {
        address: Addr,
    },
    Deposit {
        amount: u64,
    },
    FlipClaimStatus {},
    ClaimTokens {},
    Withdraw {
        denom: String,
        token_type: String,
    },
}

#[cw_serde]
pub struct AllUsersResponse {
    pub users: Vec<(String, UserInfoResponse)>, // Pair of user address and user info
}

#[cw_serde]
pub struct UserInfoResponse {
    pub address: Addr,
    pub phases: HashMap<String, PhaseInformation>,
    pub amount: u64,
    pub tokens: Uint128,
    pub claimed: bool,
}

#[cw_serde]
pub struct LaunchpadResponse {
    pub token_info: TokenInfo,
    pub buy_denom: String,
    pub buy_token_type: String,
    pub tokens_per_buy: Uint128,
    pub start_time: u64,
    pub end_time: u64,
    pub phases: Vec<PhaseResp>,
    pub claimable: bool,
    pub fees_wallet: Addr,
    pub fees_percentage: u64,
    pub withdraw_wallet: Addr,
    pub whitelist: Vec<WhitelistUser>,
}

#[cw_serde]
pub struct PhaseInfoResponse {
    pub name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub price_per_token: Uint128,
    pub supply: u64,
    pub address_list: Vec<Addr>,
    pub limit: u64,
    pub total_minted: u64,
}

// Define the query message enum
#[cw_serde]
pub enum QueryMsg {
    GetLaunchpad {},
    GetUser {
        address: Addr,
    },
    GetPhase {
        phase_name: String,
    },
}
