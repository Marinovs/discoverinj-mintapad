use std::collections::HashMap;

use cosmwasm_std::{
    to_json_binary,
    Addr,
    BalanceResponse,
    BankMsg,
    BankQuery,
    Coin,
    CosmosMsg,
    QuerierWrapper,
    QueryRequest,
    StdResult,
    Uint128,
    WasmMsg,
    WasmQuery,
};
use cw20::{ BalanceResponse as CW20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg };

use crate::state::Phase;

pub fn current_phase(current_time: u64, phases: Vec<Phase>) -> Option<Phase> {
    for phase in phases.iter() {
        if current_time >= phase.start_time && current_time <= phase.end_time {
            return Some(phase.clone());
        }
    }
    None
}

pub fn count_allowed_user_buy(addr: Addr, phases: HashMap<String, Phase>) -> u64 {
    let mut amount = 0u64;
    for (_name, phase) in phases.iter() {
        if phase.address_list.contains(&addr) {
            amount += phase.limit;
        } else if phase.address_list.len() == 0 {
            amount += phase.limit;
        }
    }

    return amount;
}

pub fn transfer_token_message(
    denom: String,
    token_type: String,
    amount: Uint128,
    receiver: Addr
) -> StdResult<CosmosMsg> {
    if token_type == "native" {
        Ok(
            (BankMsg::Send {
                to_address: receiver.clone().into(),
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount,
                }],
            }).into()
        )
    } else {
        Ok(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: denom.clone(),
                funds: vec![],
                msg: to_json_binary(
                    &(Cw20ExecuteMsg::Transfer {
                        recipient: receiver.clone().into(),
                        amount,
                    })
                )?,
            })
        )
    }
}

pub fn get_token_amount(
    querier: QuerierWrapper,
    denom: String,
    contract_addr: Addr,
    token_type: String
) -> StdResult<Uint128> {
    if token_type == "native" {
        let native_response: BalanceResponse = querier.query(
            &QueryRequest::Bank(BankQuery::Balance {
                address: contract_addr.clone().into(),
                denom: denom.clone(),
            })
        )?;
        Ok(native_response.amount.amount)
    } else {
        let balance_response: CW20BalanceResponse = querier.query(
            &QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: denom.clone(),
                msg: to_json_binary(
                    &(Cw20QueryMsg::Balance {
                        address: contract_addr.clone().into(),
                    })
                )?,
            })
        )?;
        Ok(balance_response.balance)
    }
}
