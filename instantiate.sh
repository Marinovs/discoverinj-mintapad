injectived tx wasm store artifacts/discoverinj_launchpad-aarch64.wasm \
--from=a5t \
--chain-id="injective-888" \
--gas-prices=500000000inj \
--gas=20000000 \
--yes \
--node=https://testnet.sentry.tm.injective.network:443


//  pub buy_denom: String,
    pub buy_token_type: String,
    pub tokens_per_buy: Uint128,
    pub token_info: TokenInfo,

    pub name: String,
    pub symbol: String,
    pub metadata_url: String,
    pub description: String,
    pub banner: String,

export TOKEN_INFO='{"name":"DI-TEST","symbol":"DIT","metadata_url":"test_metadata_url","description":"Test Launchpad","banner":"test_banner}'
export INST='{"admin":"inj1tx74j0uslp4pr5neyxxxgajh6gx5s9lnahpp5r","start_time":"1712001600","end_time":"1712088000","buy_denom":"inj","buy_token_type":"native","tokens_per_buy":"1",}'



injectived tx wasm instantiate $CODE_ID $INST \
--label="Test404" \
--from=$INJ_ADDRESS \
--chain-id="injective-888" \ 
--yes \
--gas-prices=500000000inj \
--gas=20000000 \
--no-admin \
--node=https://testnet.sentry.tm.injective.network:443
