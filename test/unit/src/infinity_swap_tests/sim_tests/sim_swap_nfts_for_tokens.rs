use crate::helpers::nft_functions::mint_and_approve_many;
use crate::helpers::pool_functions::prepare_pool_variations;
use crate::helpers::swap_functions::{setup_swap_test, validate_swap_fees, SwapTestSetup};
use crate::setup::setup_accounts::setup_addtl_account;
use cosmwasm_std::{StdError, StdResult, Timestamp, Uint128};
use infinity_shared::interface::{NftOrder, SwapParams, SwapResponse};
use infinity_swap::msg::QueryMsg;
use infinity_swap::state::PoolId;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_marketplace::msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg};
use sg_std::GENESIS_MINT_START_TIME;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn cant_swap_inactive_pools() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );
    let bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        false,
        0,
        0,
    );

    let nft_orders: Vec<NftOrder> = bidder_token_ids
        .to_vec()
        .drain(0..3_usize)
        .map(|token_id| NftOrder {
            token_id,
            amount: Uint128::from(100_000u128),
        })
        .collect();
    let sim_msg = QueryMsg::SimSwapNftsForTokens {
        collection: collection.to_string(),
        nft_orders,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: None,
        },
    };

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_swap.clone(), &sim_msg);

    assert!(res.unwrap().swaps.is_empty());
}

#[test]
fn can_swap_active_pools() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );
    let bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        0,
        0,
    );

    let nft_orders: Vec<NftOrder> = bidder_token_ids
        .to_vec()
        .drain(0..3_usize)
        .map(|token_id| NftOrder {
            token_id,
            amount: Uint128::from(10u128),
        })
        .collect();
    let sim_msg = QueryMsg::SimSwapNftsForTokens {
        collection: collection.to_string(),
        nft_orders,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: None,
        },
    };

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_swap.clone(), &sim_msg);

    assert!(res.is_ok());
    assert!(!res.unwrap().swaps.is_empty());
}

#[test]
fn sale_price_below_min_expected() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );
    let bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        0,
        0,
    );

    let nft_orders: Vec<NftOrder> = bidder_token_ids
        .to_vec()
        .drain(0..3_usize)
        .map(|token_id| NftOrder {
            token_id,
            amount: Uint128::from(100_000u128),
        })
        .collect();
    let sim_msg = QueryMsg::SimSwapNftsForTokens {
        collection: collection.to_string(),
        nft_orders,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: None,
        },
    };

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_swap.clone(), &sim_msg);

    assert_eq!(
            res.unwrap_err(),
            StdError::GenericErr {
                msg: "Querier contract error: Generic error: Swap error: pool sale price is below min expected"
                    .to_string()
            }
        );
}

#[test]
fn robust_query_does_not_revert_whole_tx() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );
    let bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        0,
        0,
    );

    let nft_orders: Vec<NftOrder> = bidder_token_ids
        .to_vec()
        .drain(0..3_usize)
        .map(|token_id| NftOrder {
            token_id,
            amount: Uint128::from(1_000u128),
        })
        .collect();
    let sim_msg = QueryMsg::SimSwapNftsForTokens {
        collection: collection.to_string(),
        nft_orders,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
            asset_recipient: None,
            finder: None,
        },
    };

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_swap.clone(), &sim_msg);

    assert!(res.is_ok());
    assert_eq!(res.unwrap().swaps.len(), 2);
}

#[test]
fn minimal_fee_tx_is_handled_correctly() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
        marketplace,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );
    let bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        0,
        0,
    );

    let marketplace_params: ParamsResponse = router
        .wrap()
        .query_wasm_smart(marketplace, &MarketplaceQueryMsg::Params {})
        .unwrap();
    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    let nft_orders: Vec<NftOrder> = bidder_token_ids
        .to_vec()
        .drain(0..3_usize)
        .map(|token_id| NftOrder {
            token_id,
            amount: Uint128::from(10u128),
        })
        .collect();
    let sim_msg = QueryMsg::SimSwapNftsForTokens {
        collection: collection.to_string(),
        nft_orders,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: None,
        },
    };

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_swap.clone(), &sim_msg);

    for swap in res.unwrap().swaps {
        let pool_id = swap.unpack_data::<PoolId>().unwrap().0;
        let pool = pools.iter().find(|p| p.id == pool_id).unwrap();
        validate_swap_fees(
            &swap,
            pool,
            &marketplace_params,
            &collection_info.royalty_info,
        );
    }
}

#[test]
fn finders_and_swap_fee_tx_is_handled_correctly() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
        marketplace,
        ..
    } = setup_swap_test(5000).unwrap();
    let user2 = setup_addtl_account(&mut router, "asset", 100u128).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );
    let bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let pools = prepare_pool_variations(
        &mut router,
        7,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        250,
        300,
    );

    let marketplace_params: ParamsResponse = router
        .wrap()
        .query_wasm_smart(marketplace, &MarketplaceQueryMsg::Params {})
        .unwrap();
    let collection_info: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &Sg721QueryMsg::CollectionInfo {})
        .unwrap();

    let nft_orders: Vec<NftOrder> = bidder_token_ids
        .to_vec()
        .drain(0..3_usize)
        .map(|token_id| NftOrder {
            token_id,
            amount: Uint128::from(10u128),
        })
        .collect();
    let sim_msg = QueryMsg::SimSwapNftsForTokens {
        collection: collection.to_string(),
        nft_orders,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: Some(user2.to_string()),
        },
    };

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_swap.clone(), &sim_msg);

    for swap in res.unwrap().swaps {
        let pool_id = swap.unpack_data::<PoolId>().unwrap().0;
        let pool = pools.iter().find(|p| p.id == pool_id).unwrap();
        validate_swap_fees(
            &swap,
            pool,
            &marketplace_params,
            &collection_info.royalty_info,
        );
    }
}

#[test]
fn trades_are_routed_correctly() {
    let SwapTestSetup {
        vending_template:
            MinterTemplateResponse {
                mut router,
                accts,
                collection_response_vec,
                ..
            },
        infinity_swap,
        ..
    } = setup_swap_test(5000).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let owner_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.owner,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );
    let bidder_token_ids = mint_and_approve_many(
        &mut router,
        &accts.creator,
        &accts.bidder,
        &minter,
        &collection,
        &infinity_swap,
        100,
    );

    let deposit_tokens_per_pool = Uint128::from(10_000u128);
    let _pools = prepare_pool_variations(
        &mut router,
        14,
        &None,
        &infinity_swap,
        &collection,
        &accts.owner,
        deposit_tokens_per_pool,
        owner_token_ids.to_vec(),
        6,
        true,
        0,
        0,
    );

    let num_swaps: usize = 50;
    let nft_orders: Vec<NftOrder> = bidder_token_ids
        .to_vec()
        .drain(0..num_swaps)
        .map(|token_id| NftOrder {
            token_id,
            amount: Uint128::from(10u128),
        })
        .collect();

    let sim_msg = QueryMsg::SimSwapNftsForTokens {
        collection: collection.to_string(),
        nft_orders,
        sender: accts.bidder.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: false,
            asset_recipient: None,
            finder: None,
        },
    };

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_swap.clone(), &sim_msg);
    let swaps = res.unwrap().swaps;

    assert_eq!(swaps.len(), num_swaps);
    for (idx, swap) in swaps.iter().enumerate() {
        if idx == 0 {
            continue;
        }
        assert!(swaps[idx - 1].sale_price >= swap.sale_price);
    }
}
