#[cfg(test)]
mod tests {
    use crate::contract::{ execute, instantiate };
    use crate::msg::{ AllUsersResponse, ExecuteMsg, InstantiateMsg, UserInfoResponse };
    use crate::state::{ Phase, TokenInfo, WhitelistUser, STATE, USER_INFO };
    use std::collections::HashMap;
    use std::thread;
    use std::time::Duration;

    use cosmwasm_std::testing::{ mock_dependencies_with_balance, mock_env, mock_info };
    use cosmwasm_std::{ coins, Addr, Deps, DepsMut, StdResult, Uint128 };

    // Test deposit functionality
    fn test_deposit(deps: DepsMut, address: String, amount: u64) {
        let deposit_msg = ExecuteMsg::Deposit { amount };
        let env = mock_env();
        let info = mock_info(&address, &coins((amount as u128) * 1000000000000000000, "inj")); // Adjust coin amount and denom as needed

        let res = execute(deps, env, info, deposit_msg);
        match res {
            Ok(resp) => {
                // Assert successful response attributes, messages, or data here.
                // For example:
                println!(
                    "Deposit successful in phase {} for address {}, amount {}",
                    resp.attributes.get(1).unwrap().value,
                    address,
                    amount
                );
                // You might want to check the user's token balance or the total minted amount in the phase.
            }
            Err(e) => {
                // Handle error, this could be a limit exceeded, phase timing issue, etc.
                println!(
                    "Deposit failed in phase for address {}, amount {}. Error: {:?}",
                    address,
                    amount,
                    e
                );
            }

            // Further assertions can be made here to ensure tokens are correctly allocated, etc.
        }
    }

    fn flip_claim_status(deps: DepsMut, address: String) {
        let deposit_msg = ExecuteMsg::FlipClaimStatus {};
        let env = mock_env();
        let info = mock_info(&address, &[]); // Adjust coin amount and denom as needed

        let res = execute(deps, env, info, deposit_msg);
        match res {
            Ok(response) => {
                // Assert successful response attributes, messages, or data here.
                // For example:
                println!(
                    "Claim status switched to {:?}",
                    response.attributes.get(1).unwrap().value
                );
                // You might want to check the user's token balance or the total minted amount in the phase.
            }
            Err(e) => {
                // Handle error, this could be a limit exceeded, phase timing issue, etc.
                println!("Error: {:?}", e);
            }
        }
    }

    fn claim_tokens(deps: DepsMut, address: String) {
        let claim_msg = ExecuteMsg::ClaimTokens {};
        let env = mock_env();
        let info = mock_info(&address, &[]); // Adjust coin amount and denom as needed

        let res = execute(deps, env, info, claim_msg);
        match res {
            Ok(response) => {
                // Assert successful response attributes, messages, or data here.
                // For example:
                println!(
                    "Claimed successfully {:?}",
                    response.messages
                    //response.attributes.get(1).unwrap().value,
                    //response.attributes.get(2).unwrap().value
                );
                // You might want to check the user's token balance or the total minted amount in the phase.
            }
            Err(e) => {
                // Handle error, this could be a limit exceeded, phase timing issue, etc.
                println!("Error: {:?}", e);
            }
        }
    }

    fn update_state(deps: DepsMut, address: String, denom: String) {
        let state = STATE.load(deps.storage);
        match state {
            Ok(st) => {
                let update_token_denom = ExecuteMsg::UpdateConfig {
                    start_time: None,
                    end_time: None,
                    buy_denom: None,
                    buy_token_type: None,
                    tokens_per_buy: None,
                    token_info: Some(TokenInfo {
                        name: st.token_info.name,
                        symbol: st.token_info.symbol,
                        denom: Some(denom.clone()),
                        description: st.token_info.description,
                        supply: 5000,
                    }),
                    phases: None,
                    new_admin: None,
                };
                let env = mock_env();
                let info = mock_info(&address, &[]); // Adjust coin amount and denom as needed

                let res = execute(deps, env, info, update_token_denom);
                match res {
                    Ok(response) => {
                        // Assert successful response attributes, messages, or data here.
                        // For example:
                        println!("Successfully update token denom to {}", denom);
                        // You might want to check the user's token balance or the total minted amount in the phase.
                    }
                    Err(e) => {
                        // Handle error, this could be a limit exceeded, phase timing issue, etc.
                        println!("Error: {:?}", e);
                    }

                    // Further assertions can be made here to ensure tokens are correctly allocated, etc.
                }
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }

    fn add_whitelist(deps: DepsMut, address: String, whitelist: Vec<WhitelistUser>) {
        let deposit_msg = ExecuteMsg::AddWhitelist { whitelist };

        let env = mock_env();
        let info = mock_info(&address, &[]); // Adjust coin amount and denom as needed

        let res = execute(deps, env, info, deposit_msg);
        match res {
            Ok(resp) => {
                // Assert successful response attributes, messages, or data here.
                // For example:
                println!("Added whiteelist: {}", resp.attributes.get(1).unwrap().value);
                // You might want to check the user's token balance or the total minted amount in the phase.
            }
            Err(e) => {
                // Handle error, this could be a limit exceeded, phase timing issue, etc.
                println!("Failed to add whitelist. Error: {:?}", e);
            }

            // Further assertions can be made here to ensure tokens are correctly allocated, etc.
        }
    }

    #[test]
    fn multiple_deposit_test() {
        let mut deps = mock_dependencies_with_balance(&coins(1000, "$note"));

        let phases = vec![
            Phase {
                name: "OG".to_string(),
                start_time: mock_env().block.time.seconds(),
                end_time: mock_env().block.time.seconds(),
                price_per_token: Uint128::new(1000000000000000000),
                supply: 150,
                address_list: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
                limit: 5,
                total_minted: 0,
            },
            Phase {
                name: "WL".to_string(),
                start_time: mock_env().block.time.seconds(),
                end_time: mock_env().block.time.seconds() + 20,
                price_per_token: Uint128::new(1000000000000000000),
                supply: 10000,
                address_list: vec![Addr::unchecked("addr3")],
                limit: 1200,
                total_minted: 0,
            },
            Phase {
                name: "Public".to_string(),
                start_time: mock_env().block.time.seconds() + 20,
                end_time: mock_env().block.time.seconds() + 50,
                price_per_token: Uint128::new(1000000000000000000),
                supply: 20000,
                address_list: vec![],
                limit: 1200,
                total_minted: 0,
            }
        ];

        // First, instantiate your contract
        let instantiate_msg = InstantiateMsg {
            admin: Addr::unchecked("admin"),
            start_time: mock_env().block.time.seconds(),
            end_time: mock_env().block.time.seconds() + 600,
            phases: Option::Some(phases), // You can define phases here if needed
            buy_denom: "inj".to_string(),
            buy_token_type: "native".into(),
            tokens_per_buy: Uint128::new(1),
            token_info: TokenInfo {
                name: "TEST".to_string(),
                symbol: "TEST".to_string(),
                description: "TEST".to_string(),
                denom: None,
                supply: 5000,
            },
            fees_wallet: Addr::unchecked("fees_wallet"),
            withdraw_wallet: Addr::unchecked("withdraw_wallet"),
            whitelist: Some(Vec::new()),
        };
        let instantiate_env = mock_env();
        let instantiate_info = mock_info("creator", &coins(100, "$note"));

        instantiate(deps.as_mut(), instantiate_env, instantiate_info, instantiate_msg).unwrap();

        let whitelist = vec![WhitelistUser { address: Addr::unchecked("addr1"), amount: 100u64 }];
        add_whitelist(deps.as_mut(), "admin".to_string(), whitelist);
        let state = STATE.load(deps.as_ref().storage).unwrap();
        println!("WHITELIST: {:?}", state.whitelist);

        //OG
        test_deposit(deps.as_mut(), "addr1".to_string(), 100);
        thread::sleep(Duration::from_secs(2));
        test_deposit(deps.as_mut(), "addr1".to_string(), 5);
        let state = STATE.load(deps.as_ref().storage).unwrap();
        // // let serialized_phase = serde_json::to_string_pretty(&state.phases.get(0)).unwrap();
        println!(
            "Name: {}\tTotal Minted: {}\tStart: {}\tEnd: {}\n",
            &state.phases.get(0).unwrap().name,
            &state.phases.get(0).unwrap().total_minted,
            &state.phases.get(0).unwrap().start_time,
            &state.phases.get(0).unwrap().end_time
        );

        // //thread::sleep(Duration::from_secs(8));
        // //WL
        // test_deposit(deps.as_mut(), "addr2".to_string(), 5);
        // test_deposit(deps.as_mut(), "addr3".to_string(), 5);
        // let state = STATE.load(deps.as_ref().storage).unwrap();
        // println!(
        //     "Name: {}\tTotal Minted: {}\tStart: {}\tEnd: {}\n",
        //     &state.phases.get(1).unwrap().name,
        //     &state.phases.get(1).unwrap().total_minted,
        //     &state.phases.get(1).unwrap().start_time,
        //     &state.phases.get(1).unwrap().end_time
        // );

        // //PUBLIC
        // //thread::sleep(Duration::from_secs(8));
        // test_deposit(deps.as_mut(), "addr1".to_string(), 5);
        // test_deposit(deps.as_mut(), "addr2".to_string(), 5);
        // test_deposit(deps.as_mut(), "addr2".to_string(), 5);
        // let state = STATE.load(deps.as_ref().storage).unwrap();
        // println!(
        //     "Name: {}\tTotal Minted: {}\tStart: {}\tEnd: {}\n",
        //     &state.phases.get(2).unwrap().name,
        //     &state.phases.get(2).unwrap().total_minted,
        //     &state.phases.get(2).unwrap().start_time,
        //     &state.phases.get(2).unwrap().end_time
        // );

        // test_deposit(deps.as_mut(), "addr3".to_string(), 2);

        // let all_users_response = query_all_users(deps.as_ref()).unwrap();
        // //let serialized_users = serde_json::to_string_pretty(&all_users_response.users).unwrap();
        // //println!("All Users: {}", serialized_users);

        // update_state(deps.as_mut(), "admin".to_string(), "$note".to_string());
        // flip_claim_status(deps.as_mut(), "admin".to_string());

        // claim_tokens(deps.as_mut(), "admin".to_string());
    }
}
