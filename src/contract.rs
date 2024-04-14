use std::collections::HashMap;

use cosmwasm_std::{
    attr,
    entry_point,
    to_json_binary,
    Addr,
    Binary,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
    Uint128,
};

use crate::{
    msg::{
        ExecuteMsg,
        InstantiateMsg,
        LaunchpadResponse,
        PhaseInfoResponse,
        QueryMsg,
        UserInfoResponse,
    },
    state::{
        Phase,
        PhaseInformation,
        PhaseResp,
        State,
        TokenInfo,
        User,
        UserToken,
        WhitelistUser,
        STATE,
        USERS,
        USER_INFO,
    },
    utils::{ current_phase, get_token_amount, transfer_token_message },
};

// version info for migration info
//const CONTRACT_NAME: &str = "Discoverinj-Launchpad";
//const CONTRACT_VERSION: &str = "1.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg
) -> StdResult<Response> {
    let state = State {
        admin: msg.admin,
        start_time: msg.start_time,
        end_time: msg.end_time,
        phases: msg.phases.unwrap_or_else(Vec::new),
        buy_denom: msg.buy_denom,
        buy_token_type: msg.buy_token_type,
        tokens_per_buy: msg.tokens_per_buy,
        token_info: msg.token_info,
        claimable: false,
        fees_wallet: msg.fees_wallet,
        fees_percentage: 0,
        withdraw_wallet: msg.withdraw_wallet,
        whitelist: msg.whitelist.unwrap_or_default(),
    };

    if msg.start_time > msg.end_time {
        return Err(StdError::generic_err("Start time must be before end time"));
    }

    USERS.save(deps.storage, "user_tokens".to_string(), &Vec::new())?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            start_time,
            end_time,
            buy_denom,
            buy_token_type,
            tokens_per_buy,
            token_info,
            phases,
            new_admin,
        } =>
            update_config(
                deps,
                env,
                info,
                start_time,
                end_time,
                buy_denom,
                buy_token_type,
                tokens_per_buy,
                token_info,
                phases,
                new_admin
            ),
        ExecuteMsg::AddPhase { phase } => add_phase(deps, env, info, phase),
        ExecuteMsg::UpdatePhase {
            name,
            start_time,
            end_time,
            price_per_token,
            supply,
            address_list,
            limit,
        } =>
            update_phase(
                deps,
                env,
                info,
                name,
                start_time,
                end_time,
                price_per_token,
                supply,
                address_list,
                limit
            ),
        ExecuteMsg::RemovePhase { name } => remove_phase(deps, env, info, name),
        ExecuteMsg::AddWhitelist { whitelist } => add_whitelist(deps, env, info, whitelist),
        ExecuteMsg::RemoveFromWhitelist { address } =>
            remove_from_whitelist(deps, env, info, address),
        ExecuteMsg::Deposit { amount } => deposit(deps, env, info, amount),
        ExecuteMsg::FlipClaimStatus {} => flip_claim_status(deps, env, info),
        ExecuteMsg::ClaimTokens {} => claim_token(deps, env, info),
        ExecuteMsg::Withdraw { denom, token_type } => withdraw(deps, env, info, denom, token_type),
    }
}

fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Option<u64>,
    end_time: Option<u64>,
    buy_denom: Option<String>,
    buy_token_type: Option<String>,
    tokens_per_buy: Option<Uint128>,
    token_info: Option<TokenInfo>,
    phases: Option<Vec<Phase>>,
    new_admin: Option<Addr>
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;

    if state.admin != info.sender {
        return Err(StdError::generic_err("Unauthorized: not admin"));
    }

    if env.block.time.seconds() > state.start_time && env.block.time.seconds() < state.end_time {
        return Err(StdError::generic_err("Unauthorized: launchpad already in progress"));
    }

    if start_time > end_time {
        return Err(StdError::generic_err("Start time must be before end time"));
    }

    if buy_denom.is_some() {
        state.buy_denom = buy_denom.unwrap();
    }

    if buy_token_type.is_some() {
        if buy_token_type.clone().unwrap() == "native" || buy_token_type.clone().unwrap() == "cw20" {
            state.buy_token_type = buy_token_type.unwrap();
        }
    }

    if tokens_per_buy.is_some() {
        state.tokens_per_buy = tokens_per_buy.unwrap();
    }

    if token_info.is_some() {
        state.token_info = token_info.unwrap();
    }

    if start_time.is_some() {
        state.start_time = start_time.unwrap();
    }

    if end_time.is_some() {
        state.end_time = end_time.unwrap();
    }

    if phases.is_some() {
        state.phases = phases.unwrap();
    }

    if new_admin.is_some() {
        state.admin = new_admin.unwrap();
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

fn add_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    whitelist: Vec<WhitelistUser>
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    if state.admin != info.sender {
        return Err(StdError::generic_err("Unauthorized: not admin"));
    }

    // Launchpad start check
    if state.is_launchpad_started(env) {
        return Err(StdError::generic_err("Unauthorized: Launchpad started"));
    }

    let mut attributes = vec![];
    // Iterate over the new whitelist entries
    for wl in whitelist {
        // Check if the address is already in the whitelist
        let mut found = false;
        for item in &mut state.whitelist {
            if item.address == wl.address {
                // Update the existing entry with the new value
                item.amount = wl.amount;
                found = true;
                break;
            }
        }

        // If the address was not found, add it as a new entry
        if !found {
            state.whitelist.push(WhitelistUser { address: wl.address.clone(), amount: wl.amount });
            attributes.push(("address, amount", format!("{} {}", wl.address, wl.amount)));
        }
    }

    // Save the updated state
    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("action", "add_whitelist").add_attributes(attributes))
}

fn remove_from_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: Addr
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    if state.admin != info.sender {
        return Err(StdError::generic_err("Unauthorized: not admin"));
    }

    // Launchpad start check
    if state.is_launchpad_started(env) {
        return Err(StdError::generic_err("Unauthorized: Launchpad started"));
    }
    // Attempt to remove the address from the whitelist
    let was_present = state.whitelist.iter().position(|item| item.address == &address);

    let response = Response::new().add_attribute("action", "remove_from_whitelist");

    if let Some(index) = was_present {
        state.whitelist.remove(index);
        STATE.save(deps.storage, &state)?;
        Ok(response.add_attribute("removed_address", address.to_string()))
    } else {
        Ok(response.add_attribute("address_not_found", address.to_string()))
    }
}

fn deposit(deps: DepsMut, env: Env, info: MessageInfo, amount: u64) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    let current_time = env.block.time.seconds();

    // Check if we're within the launchpad time
    if !(current_time >= state.start_time && current_time <= state.end_time) {
        return Err(StdError::generic_err("Not in launchpad time"));
    }

    // Attempt to load user, or initialize a new one if not found
    let mut usr = USER_INFO.may_load(deps.storage, info.sender.clone())?.unwrap_or(User {
        buy_phases: HashMap::new(),
        amount: 0,
        tokens: Uint128::zero(),
        claimed: false,
    });

    // Determine the current phase, return an error if not found or not in phase time
    let mut current_phase = current_phase(current_time, state.phases.clone()).ok_or_else(||
        StdError::generic_err("Unauthorized: Not in launchpad time")
    )?;

    let previous_phase = get_previous_phase(&current_phase, &state.phases.clone());
    if previous_phase.is_some() && current_phase.total_minted == 0 {
        let pr_phase = previous_phase.unwrap();
        current_phase.supply += pr_phase.supply - pr_phase.total_minted;
        if current_phase.address_list.len() > 0 {
            current_phase.address_list.extend(pr_phase.address_list.iter().cloned());
        }

        for phase in state.phases.iter_mut() {
            if phase.name == current_phase.name {
                phase.supply += pr_phase.supply - pr_phase.total_minted;
                if phase.address_list.len() > 0 {
                    phase.address_list.extend(pr_phase.address_list.iter().cloned());
                }
            }
        }
    }

    // Perform checks to validate the deposit
    validate_deposit(&info, &state, amount, &mut current_phase, &usr)?;

    // Record the deposit in the user's account and phase
    record_deposit(&state, amount, &mut usr, &current_phase)?;

    // if success, update total mint
    for phase in state.phases.iter_mut() {
        if phase.name == current_phase.name {
            // Directly modify the found phase
            phase.total_minted += amount;
            break; // Exit the loop once the phase is found and updated
        }
    }

    // Load the existing user tokens from storage
    let mut user_tokens = USERS.load(deps.storage, "user_tokens".to_string())?;

    let mut found = false;
    for token in user_tokens.iter_mut() {
        if token.address == info.sender {
            token.tokens += usr.tokens;
            found = true;
            break;
        }
    }

    if !found {
        user_tokens.push(UserToken {
            address: info.sender.clone(),
            tokens: usr.tokens,
        });
    }

    USERS.save(deps.storage, "user_tokens".to_string(), &user_tokens)?;
    STATE.save(deps.storage, &state)?;
    USER_INFO.save(deps.storage, info.sender, &usr)?;

    Ok(
        Response::new()
            .add_attribute("action", "deposit")
            .add_attribute("current_phase", current_phase.name)
            .add_attribute("amount_bought", amount.to_string())
    )
}

fn validate_deposit(
    info: &MessageInfo,
    state: &State,
    amount: u64,
    current_phase: &mut Phase,
    usr: &User
) -> StdResult<()> {
    if !current_phase.address_list.is_empty() && !current_phase.address_list.contains(&info.sender) {
        return Err(StdError::generic_err("Unauthorized: Not in this phase"));
    }

    // Check phase supply limits
    if current_phase.total_minted + amount > current_phase.supply {
        return Err(StdError::generic_err("Invalid amount: overbuy"));
    }

    let wl_user = state.whitelist
        .iter()
        .find(|user| &user.address == info.sender)
        .map_or(0, |user| user.amount);

    // Check individual buy limit
    if (amount > current_phase.limit || amount <= 0) && wl_user < amount {
        return Err(
            StdError::generic_err(
                format!(
                    "Invalid amount, tried to buy {} over {} limit",
                    amount,
                    current_phase.limit
                )
            )
        );
    }

    // Check if user has already reached the limit for this phase
    if
        *usr.buy_phases.get(&current_phase.name).unwrap_or(&0) + amount > current_phase.limit &&
        wl_user < amount
    {
        return Err(
            StdError::generic_err(format!("Max buy phase ({}) reached", current_phase.name))
        );
    }

    // Check payment
    if
        info.funds.is_empty() ||
        info.funds[0].denom != state.buy_denom ||
        info.funds[0].amount != current_phase.price_per_token * Uint128::from(amount)
    {
        return Err(
            StdError::generic_err(
                format!(
                    "Payment Failed, expected {}, got {}",
                    current_phase.price_per_token * Uint128::from(amount),
                    info.funds[0].amount
                )
            )
        );
    }

    Ok(())
}

fn get_previous_phase(current_phase: &Phase, all_phases: &Vec<Phase>) -> Option<Phase> {
    // Find the index of the current phase
    if
        let Some(current_index) = all_phases
            .clone()
            .iter()
            .position(|phase| phase.name == current_phase.name)
    {
        // Check if there is a previous phase
        if current_index > 0 {
            // Return the previous phase
            return all_phases.get(current_index - 1).cloned();
        }
    }
    // If no previous phase exists or current phase not found, return None
    None
}

fn record_deposit(
    state: &State,
    amount: u64,
    usr: &mut User,
    current_phase: &Phase
) -> StdResult<()> {
    usr.amount += amount;
    *usr.buy_phases.entry(current_phase.name.clone()).or_insert(0) += amount;
    usr.tokens += Uint128::from(amount) * state.tokens_per_buy;
    // Assuming current_phase is a mutable reference if you want to update total_minted here, you'd need to adjust the function signature or manage this outside.
    Ok(())
}

fn add_phase(deps: DepsMut, env: Env, info: MessageInfo, phase: Phase) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;

    // Authorization check
    if state.admin != info.sender {
        return Err(StdError::generic_err("Unauthorized: not admin"));
    }

    // Launchpad start check
    if state.is_launchpad_started(env) {
        return Err(StdError::generic_err("Unauthorized: Launchpad started"));
    }

    // Validate the new phase
    validate_phase(&phase)?;

    // Check for existing phase with the same name
    if state.phases.iter().any(|p| p.name == phase.name) {
        return Err(StdError::generic_err("Phase already exists"));
    }

    // Add the new phase
    state.phases.push(phase.clone());
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("action", "add_phase").add_attribute("phase_name", phase.name))
}

fn update_phase(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    start_time: Option<u64>,
    end_time: Option<u64>,
    price_per_token: Option<Uint128>,
    supply: Option<u64>,
    address_list: Option<Vec<Addr>>,
    limit: Option<u64>
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    // Authorization check
    if state.admin != info.sender {
        return Err(StdError::generic_err("Unauthorized: not admin"));
    }

    // Launchpad start check
    if state.is_launchpad_started(env) {
        return Err(StdError::generic_err("Unauthorized: Launchpad started"));
    }

    // Find and update the phase
    let phase = state.phases
        .iter_mut()
        .find(|p| p.name == name)
        .ok_or_else(|| StdError::generic_err("Phase does not exist"))?;

    // Apply updates
    if let Some(start) = start_time {
        phase.start_time = start;
    }
    if let Some(end) = end_time {
        phase.end_time = end;
    }
    if let Some(price) = price_per_token {
        phase.price_per_token = price;
    }
    if let Some(supply) = supply {
        phase.supply = supply;
    }
    if let Some(list) = address_list {
        phase.address_list = list;
    }
    if let Some(limit) = limit {
        phase.limit = limit;
    }

    // Validate the updated phase
    validate_phase(phase)?;

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("action", "update_phase").add_attribute("phase_name", name))
}

fn remove_phase(deps: DepsMut, env: Env, info: MessageInfo, name: String) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;

    // Authorization check
    if state.admin != info.sender {
        return Err(StdError::generic_err("Unauthorized: not admin"));
    }

    // Launchpad start check
    if state.is_launchpad_started(env) {
        return Err(StdError::generic_err("Unauthorized: Launchpad started"));
    }

    // Remove the phase
    let initial_len = state.phases.len();
    state.phases.retain(|p| p.name != name);
    if state.phases.len() == initial_len {
        return Err(StdError::generic_err("Phase doesn't exist"));
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("action", "remove_phase").add_attribute("phase_name", name))
}

// Utility function for checking if the launchpad has started
impl State {
    fn is_launchpad_started(&self, env: Env) -> bool {
        let current_time = env.block.time.seconds(); // Adjust based on actual env usage
        current_time > self.start_time
    }
}

// Phase validation before adding/updating
fn validate_phase(phase: &Phase) -> StdResult<()> {
    if phase.start_time > phase.end_time {
        Err(StdError::generic_err("Start time must be before end time"))
    } else if phase.supply == 0 {
        Err(StdError::generic_err("Max deposit must be > 0"))
    } else if phase.price_per_token.is_zero() {
        Err(StdError::generic_err("Price must be > 0"))
    } else {
        Ok(())
    }
}

fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    token_type: String
) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    if state.admin != info.sender {
        return Err(StdError::generic_err("Unauthorized: not admin"));
    }

    let amount_raised = get_token_amount(
        deps.querier,
        denom.clone(),
        env.contract.address,
        token_type.clone()
    )?;

    let msg = transfer_token_message(denom.clone(), token_type, amount_raised, info.sender)?;

    Ok(Response::new().add_message(msg))
}

fn flip_claim_status(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    if state.admin != info.sender {
        return Err(StdError::generic_err("Unauthorized: not admin"));
    }

    state.claimable = !state.claimable;

    if state.claimable && (state.token_info.denom.is_none() || state.token_info.decimals.is_none()) {
        return Err(StdError::generic_err("Token denom not settled"));
    }

    STATE.save(deps.storage, &state)?;

    Ok(
        Response::new()
            .add_attribute("action", "flip_claim_status")
            .add_attribute("is_claimable", state.claimable.to_string())
    )
}

fn claim_token(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    if !state.claimable {
        return Err(StdError::generic_err("Not claimble"));
    }

    let mut usr = USER_INFO.load(deps.storage, info.sender.clone()).map_err(|_|
        StdError::generic_err("User not found")
    )?;

    if usr.claimed == true {
        return Err(StdError::generic_err("Already claimed"));
    }

    let token_transfer_msg = transfer_token_message(
        state.token_info.denom.clone().unwrap(),
        "cw20".to_string(),
        Uint128::from(u64::pow(10, state.token_info.decimals.unwrap() as u32)) * usr.tokens,
        info.sender.clone()
    )?;
    usr.claimed = true;
    USER_INFO.save(deps.storage, info.sender, &usr)?;
    Ok(Response::new().add_message(token_transfer_msg).add_attribute("action", "claim_airdrop"))
}

// fn airdrop_tokens(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
//     let state = STATE.load(deps.storage)?;
//     if state.admin != info.sender {
//         return Err(StdError::generic_err("Unauthorized: not admin"));
//     }
//     if !state.claimable {
//         return Err(StdError::generic_err("Not in claimable period"));
//     }
//     let users = USERS.load(deps.storage, "user_tokens".to_string());
//     let mut msgs = Vec::new();
//     for usr in users.unwrap().iter() {
//         let mut user = USER_INFO.load(deps.storage, Addr::unchecked(usr.address.clone()))?;
//         if user.claimed == true {
//             continue;
//         }

//         let token_transfer_msg = transfer_token_message(
//             state.token_info.denom.clone().unwrap(),
//             "cw20".to_string(),
//             Uint128::from(u64::pow(10, state.token_info.decimals.unwrap() as u32)) * usr.tokens,
//             Addr::unchecked(usr.address.clone())
//         )?;
//         msgs.push(token_transfer_msg);
//         user.claimed = true;
//         USER_INFO.save(deps.storage, Addr::unchecked(usr.address.clone()), &user)?;
//     }

//     Ok(Response::new().add_messages(msgs).add_attribute("action", "execute_airdrop"))
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLaunchpad {} => to_json_binary(&query_launchpad(deps, env)?),
        QueryMsg::GetUser { address } => to_json_binary(&query_user(deps, env, address)?),
        QueryMsg::GetPhase { phase_name } => to_json_binary(&query_phase(deps, env, phase_name)?),
    }
}

fn query_launchpad(deps: Deps, _env: Env) -> StdResult<LaunchpadResponse> {
    let state = STATE.load(deps.storage)?;
    let mut new_ph = Vec::new();
    for ph in state.phases.iter() {
        new_ph.push(PhaseResp {
            name: ph.name.clone(),
            start_time: ph.start_time,
            end_time: ph.end_time,
            price_per_token: ph.price_per_token,
            supply: ph.supply,
            address_list: ph.address_list.len() as u64,
            limit: ph.limit,
            total_minted: ph.total_minted,
        });
    }
    Ok(LaunchpadResponse {
        token_info: state.token_info,
        buy_denom: state.buy_denom,
        buy_token_type: state.buy_token_type,
        tokens_per_buy: state.tokens_per_buy,
        start_time: state.start_time,
        end_time: state.end_time,
        phases: new_ph,
        claimable: state.claimable,
        fees_wallet: state.fees_wallet,
        fees_percentage: state.fees_percentage,
        withdraw_wallet: state.withdraw_wallet,
        whitelist: state.whitelist,
    })
}

fn query_user(deps: Deps, _env: Env, address: Addr) -> StdResult<UserInfoResponse> {
    let user = USER_INFO.load(deps.storage, address.clone()).unwrap_or_else(|_| User {
        buy_phases: HashMap::new(),
        amount: 0,
        tokens: Uint128::zero(),
        claimed: false,
    });

    let mut phases = HashMap::new();

    let state = STATE.load(deps.storage)?;

    for ph in state.phases.iter() {
        let mut is_eligible = ph.address_list
            .iter()
            .find(|&addr| addr == address)
            .is_some();

        if ph.address_list.len() == 0 {
            is_eligible = true;
        }

        let current_mint = user.buy_phases
            .iter()
            .find(|&ub| *ub.0 == ph.name)
            .map(|ub| ub.1)
            .unwrap_or(&0);

        phases.insert(ph.name.to_string(), PhaseInformation {
            limit: ph.limit,
            current_mint: current_mint.clone(),
            eligible: is_eligible,
        });
    }

    Ok(UserInfoResponse {
        address,
        phases,
        amount: user.amount,
        tokens: user.tokens,
        claimed: user.claimed,
    })
}

fn query_phase(deps: Deps, _env: Env, phase_name: String) -> StdResult<PhaseInfoResponse> {
    let state = STATE.load(deps.storage)?;

    // Find the phase with the matching name
    let phase = state.phases.iter().find(|&p| p.name == phase_name);

    match phase {
        Some(phase) => {
            Ok(PhaseInfoResponse {
                name: phase.name.clone(), // Assuming PhaseInfoResponse expects a name field
                start_time: phase.start_time,
                end_time: phase.end_time,
                price_per_token: phase.price_per_token,
                supply: phase.supply,
                address_list: phase.address_list.clone(),
                limit: phase.limit,
                total_minted: phase.total_minted,
            })
        }
        None => Err(StdError::generic_err("Phase not found")),
    }
}
