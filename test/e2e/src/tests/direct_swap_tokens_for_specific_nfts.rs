use crate::helpers::{
    chain::Chain,
    helper::{gen_users, latest_block_time},
    instantiate::instantiate_minter,
    pool::{create_pools_from_fixtures, pool_execute_message, pool_query_message},
};
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosmwasm_std::Uint128;
use infinity_swap::msg::{
    ExecuteMsg as InfinitySwapExecuteMsg, NftSwap, NftTokenIdsResponse,
    QueryMsg as InfinitySwapQueryMsg, QueryOptions, SwapParams, SwapResponse,
};
use test_context::test_context;

#[test_context(Chain)]
#[test]
#[ignore]
fn swap_small(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let prefix = chain.cfg.orc_cfg.chain_cfg.prefix.clone();
    let master_account = chain.cfg.users[1].clone();

    let pool_deposit_amount = 1_000_000;
    let balance = 1_000_000_000;
    let mut users = gen_users(chain, 2, balance);
    let maker = users.pop().unwrap();
    let maker_addr = maker.to_addr(&prefix).unwrap();
    let taker = users.pop().unwrap();
    let taker_addr = taker.to_addr(&prefix).unwrap();

    let asset_account = gen_users(chain, 1, 1)[0].clone();
    let asset_account_addr = asset_account.to_addr(&prefix).unwrap();

    // init minter
    instantiate_minter(
        &mut chain.orc,
        // set creator address as maker to allow for minting on base minter
        maker_addr.to_string(),
        &master_account.key,
        &denom,
    )
    .unwrap();

    let pools = create_pools_from_fixtures(
        chain,
        &maker,
        pool_deposit_amount,
        10,
        &Some(asset_account_addr.to_string()),
        150,
        300,
    );

    for pool in pools.iter() {
        if !pool.can_sell_nfts() {
            continue;
        }
        let num_swaps = 3u8;

        let nft_token_ids_res: NftTokenIdsResponse = pool_query_message(
            chain,
            InfinitySwapQueryMsg::PoolNftTokenIds {
                pool_id: pool.id,
                query_options: QueryOptions {
                    descending: None,
                    start_after: None,
                    limit: Some(num_swaps as u32),
                },
            },
        );

        let nfts_to_swap_for: Vec<NftSwap> = nft_token_ids_res
            .nft_token_ids
            .clone()
            .into_iter()
            .map(|token_id| NftSwap {
                nft_token_id: token_id,
                token_amount: Uint128::from(1_000_000u128),
            })
            .collect();

        let sim_res: SwapResponse = pool_query_message(
            chain,
            InfinitySwapQueryMsg::SimDirectSwapTokensForSpecificNfts {
                pool_id: pool.id,
                nfts_to_swap_for: nfts_to_swap_for.clone(),
                sender: taker_addr.to_string(),
                swap_params: SwapParams {
                    deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                    robust: false,
                    asset_recipient: None,
                    finder: None,
                },
            },
        );
        assert!(!sim_res.swaps.is_empty());

        let total_amount = nfts_to_swap_for
            .iter()
            .fold(Uint128::zero(), |acc, nft_swap| acc + nft_swap.token_amount);

        let exec_resp = pool_execute_message(
            chain,
            InfinitySwapExecuteMsg::DirectSwapTokensForSpecificNfts {
                pool_id: pool.id,
                nfts_to_swap_for,
                swap_params: SwapParams {
                    deadline: latest_block_time(&chain.orc).plus_seconds(1_000),
                    robust: false,
                    asset_recipient: None,
                    finder: None,
                },
            },
            "infinity-swap-direct-swap-tokens-for-specific-nfts",
            vec![OrcCoin {
                amount: total_amount.u128(),
                denom: denom.parse().unwrap(),
            }],
            &taker,
        );

        let tags = exec_resp
            .res
            .find_event_tags("wasm-swap".to_string(), "pool_id".to_string());
        assert!(tags.len() == num_swaps as usize);
    }
}
