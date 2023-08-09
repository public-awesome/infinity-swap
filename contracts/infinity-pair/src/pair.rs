use crate::error::ContractError;
use crate::helpers::PayoutContext;
use crate::math::{
    calc_cp_trade_buy_from_pair_price, calc_cp_trade_sell_to_pair_price,
    calc_exponential_spot_price_user_submits_nft, calc_exponential_spot_price_user_submits_tokens,
    calc_exponential_trade_buy_from_pair_price, calc_linear_spot_price_user_submits_nft,
    calc_linear_spot_price_user_submits_tokens, calc_linear_trade_buy_from_pair_price,
};
use crate::msg::TransactionType;
use crate::state::{
    BondingCurve, PairConfig, PairImmutable, PairInternal, PairType, QuoteSummary, PAIR_CONFIG,
    PAIR_IMMUTABLE, PAIR_INTERNAL,
};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, to_binary, Addr, Decimal, Storage, Uint128, WasmMsg};
use infinity_index::msg::ExecuteMsg as InfinityIndexExecuteMsg;
use sg_marketplace_common::address::address_or;
use sg_marketplace_common::coin::transfer_coins;
use sg_std::Response;
use stargaze_fair_burn::append_fair_burn_msg;

impl QuoteSummary {
    pub fn total(&self) -> Uint128 {
        self.fair_burn.amount
            + self.royalty.as_ref().map_or(Uint128::zero(), |p| p.amount)
            + self.swap.as_ref().map_or(Uint128::zero(), |p| p.amount)
            + self.seller_amount
    }

    pub fn payout(
        &self,
        denom: &String,
        seller_recipient: &Addr,
        mut response: Response,
    ) -> Result<Response, ContractError> {
        response = append_fair_burn_msg(
            &self.fair_burn.recipient,
            vec![coin(self.fair_burn.amount.u128(), denom)],
            None,
            response,
        );

        if let Some(royalty) = &self.royalty {
            response = transfer_coins(
                vec![coin(royalty.amount.u128(), denom)],
                &royalty.recipient,
                response,
            );
        }

        if let Some(swap) = &self.swap {
            response =
                transfer_coins(vec![coin(swap.amount.u128(), denom)], &swap.recipient, response);
        }

        response = transfer_coins(
            vec![coin(self.seller_amount.u128(), denom)],
            seller_recipient,
            response,
        );

        Ok(response)
    }
}

#[cw_serde]
pub struct Pair {
    pub immutable: PairImmutable<Addr>,
    pub config: PairConfig<Addr>,
    pub internal: PairInternal,
    pub total_tokens: Uint128,
}

impl Pair {
    pub fn initialize(
        storage: &mut dyn Storage,
        immutable: PairImmutable<Addr>,
        config: PairConfig<Addr>,
    ) -> Result<Self, ContractError> {
        PAIR_IMMUTABLE.save(storage, &immutable)?;

        Ok(Pair::new(
            immutable,
            config,
            PairInternal {
                total_nfts: 0u64,
                buy_from_pair_quote_summary: None,
                sell_to_pair_quote_summary: None,
            },
            Uint128::zero(),
        ))
    }

    pub fn new(
        immutable: PairImmutable<Addr>,
        config: PairConfig<Addr>,
        internal: PairInternal,
        total_tokens: Uint128,
    ) -> Self {
        Self {
            immutable,
            config,
            internal,
            total_tokens,
        }
    }

    pub fn save_and_update_indices(
        &mut self,
        storage: &mut dyn Storage,
        payout_context: &PayoutContext,
        mut response: Response,
    ) -> Result<Response, ContractError> {
        self.update_sell_to_pair_quote_summary(payout_context);
        self.update_buy_from_pair_quote_summary(payout_context);

        PAIR_CONFIG.save(storage, &self.config)?;
        PAIR_INTERNAL.save(storage, &self.internal)?;

        response = self.update_index(&payout_context.global_config.infinity_index, response);

        Ok(response)
    }

    pub fn asset_recipient(&self) -> Addr {
        address_or(self.config.asset_recipient.as_ref(), &self.immutable.owner)
    }

    pub fn reinvest_nfts(&self) -> bool {
        match self.config.pair_type {
            PairType::Trade {
                reinvest_nfts,
                ..
            } => reinvest_nfts,
            _ => false,
        }
    }

    pub fn reinvest_tokens(&self) -> bool {
        match self.config.pair_type {
            PairType::Trade {
                reinvest_tokens,
                ..
            } => reinvest_tokens,
            _ => false,
        }
    }

    pub fn swap_fee_percent(&self) -> Decimal {
        match self.config.pair_type {
            PairType::Trade {
                swap_fee_percent,
                ..
            } => swap_fee_percent,
            _ => Decimal::zero(),
        }
    }

    pub fn swap_nft_for_tokens(&mut self) {
        self.total_tokens -= self.internal.sell_to_pair_quote_summary.as_ref().unwrap().total();

        if self.reinvest_nfts() {
            self.internal.total_nfts += 1u64;
        };

        self.update_spot_price(TransactionType::UserSubmitsNfts);
    }

    pub fn sim_swap_nft_for_tokens(&mut self, payout_context: &PayoutContext) {
        self.swap_nft_for_tokens();
        self.update_sell_to_pair_quote_summary(payout_context);
        self.update_buy_from_pair_quote_summary(payout_context);
    }

    pub fn swap_tokens_for_nft(&mut self) {
        self.internal.total_nfts -= 1u64;

        if self.reinvest_tokens() {
            self.total_tokens +=
                self.internal.buy_from_pair_quote_summary.as_ref().unwrap().seller_amount;
        };

        self.update_spot_price(TransactionType::UserSubmitsTokens);
    }

    pub fn sim_swap_tokens_for_nft(&mut self, payout_context: &PayoutContext) {
        self.swap_tokens_for_nft();
        self.update_sell_to_pair_quote_summary(payout_context);
        self.update_buy_from_pair_quote_summary(payout_context);
    }

    fn update_spot_price(&mut self, tx_type: TransactionType) {
        match self.config.bonding_curve {
            BondingCurve::Linear {
                spot_price,
                delta,
            } => {
                let result = match tx_type {
                    TransactionType::UserSubmitsNfts => {
                        calc_linear_spot_price_user_submits_nft(spot_price, delta)
                    },
                    TransactionType::UserSubmitsTokens => {
                        calc_linear_spot_price_user_submits_tokens(spot_price, delta)
                    },
                };
                match result {
                    Ok(new_spot_price) => {
                        self.config.bonding_curve = BondingCurve::Linear {
                            spot_price: new_spot_price,
                            delta,
                        };
                    },
                    Err(_e) => {
                        self.config.is_active = false;
                    },
                }
            },
            BondingCurve::Exponential {
                spot_price,
                delta,
            } => {
                let result = match tx_type {
                    TransactionType::UserSubmitsNfts => {
                        calc_exponential_spot_price_user_submits_nft(spot_price, delta)
                    },
                    TransactionType::UserSubmitsTokens => {
                        calc_exponential_spot_price_user_submits_tokens(spot_price, delta)
                    },
                };
                match result {
                    Ok(new_spot_price) => {
                        self.config.bonding_curve = BondingCurve::Exponential {
                            spot_price: new_spot_price,
                            delta,
                        };
                    },
                    Err(_e) => {
                        self.config.is_active = false;
                    },
                }
            },
            BondingCurve::ConstantProduct => {},
        };
    }

    fn update_sell_to_pair_quote_summary(&mut self, payout_context: &PayoutContext) {
        if !self.config.is_active {
            self.internal.sell_to_pair_quote_summary = None;
            return;
        }

        let sale_amount_option = match self.config.bonding_curve {
            BondingCurve::Linear {
                spot_price,
                ..
            }
            | BondingCurve::Exponential {
                spot_price,
                ..
            } => Some(spot_price),
            BondingCurve::ConstantProduct => {
                calc_cp_trade_sell_to_pair_price(self.total_tokens, self.internal.total_nfts).ok()
            },
        };

        self.internal.sell_to_pair_quote_summary = match sale_amount_option {
            Some(sale_amount) if sale_amount <= self.total_tokens => {
                payout_context.build_quote_summary(&self, sale_amount)
            },
            _ => None,
        };
    }

    fn update_buy_from_pair_quote_summary(&mut self, payout_context: &PayoutContext) {
        if !self.config.is_active || self.internal.total_nfts == 0u64 {
            self.internal.buy_from_pair_quote_summary = None;
            return;
        }

        let sale_amount_option = match (&self.config.pair_type, &self.config.bonding_curve) {
            (
                PairType::Nft,
                BondingCurve::Linear {
                    spot_price,
                    ..
                }
                | BondingCurve::Exponential {
                    spot_price,
                    ..
                },
            ) => Some(*spot_price),
            (
                PairType::Trade {
                    ..
                },
                BondingCurve::Linear {
                    spot_price,
                    delta,
                },
            ) => calc_linear_trade_buy_from_pair_price(*spot_price, *delta).ok(),
            (
                PairType::Trade {
                    ..
                },
                BondingCurve::Exponential {
                    spot_price,
                    delta,
                },
            ) => calc_exponential_trade_buy_from_pair_price(*spot_price, *delta).ok(),
            (
                PairType::Trade {
                    ..
                },
                BondingCurve::ConstantProduct,
            ) => {
                calc_cp_trade_buy_from_pair_price(self.total_tokens, self.internal.total_nfts).ok()
            },
            _ => None,
        };

        self.internal.buy_from_pair_quote_summary = match sale_amount_option {
            Some(sale_amount) => payout_context.build_quote_summary(&self, sale_amount),
            _ => None,
        };
    }

    fn update_index(&self, infinity_index: &Addr, response: Response) -> Response {
        let sell_to_pair_quote =
            self.internal.sell_to_pair_quote_summary.as_ref().map(|summary| summary.seller_amount);

        let buy_from_pair_quote =
            self.internal.sell_to_pair_quote_summary.as_ref().map(|summary| summary.total());

        response.add_message(WasmMsg::Execute {
            contract_addr: infinity_index.to_string(),
            msg: to_binary(&InfinityIndexExecuteMsg::UpdatePairIndices {
                collection: self.immutable.collection.to_string(),
                denom: self.immutable.denom.clone(),
                sell_to_pair_quote,
                buy_from_pair_quote,
            })
            .unwrap(),
            funds: vec![],
        })
    }
}
