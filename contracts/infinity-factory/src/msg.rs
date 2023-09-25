use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint128};
use infinity_pair::pair::Pair;
use infinity_pair::state::{PairConfig, PairImmutable};
use sg_index_query::QueryOptions;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the infinity global contract
    pub infinity_global: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreatePair {
        /// The immutable parameters of the pair
        pair_immutable: PairImmutable<String>,
        /// The user configurable parameters of the pair
        pair_config: PairConfig<String>,
    },
    CreatePair2 {
        /// The immutable parameters of the pair
        pair_immutable: PairImmutable<String>,
        /// The user configurable parameters of the pair
        pair_config: PairConfig<String>,
    },
}

#[cw_serde]
pub struct NextPairResponse {
    pub sender: Addr,
    pub code_id: u64,
    pub counter: u64,
    pub salt: Binary,
    pub pair: Addr,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(NextPairResponse)]
    NextPair {
        sender: String,
    },
    #[returns(Vec<(u64, Addr)>)]
    PairsByOwner {
        owner: String,
        code_id: u64,
        query_options: Option<QueryOptions<u64>>,
    },
    #[returns(QuotesResponse)]
    SimSellToPairQuotes {
        pair: Pair,
        limit: u32,
    },
    #[returns(QuotesResponse)]
    SimBuyFromPairQuotes {
        pair: Pair,
        limit: u32,
    },
}

#[cw_serde]
pub struct QuotesResponse {
    pub denom: String,
    pub quotes: Vec<Uint128>,
}
