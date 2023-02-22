/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Addr = string;
export interface ConfigResponse {
  config: Config;
}
export interface Config {
  denom: string;
  developer?: Addr | null;
  marketplace_addr: Addr;
}
export interface InstantiateMsg {
  denom: string;
  developer?: string | null;
  marketplace_addr: string;
}
export type Uint128 = string;
export interface NftSwap {
  nft_token_id: string;
  token_amount: Uint128;
}
export type BondingCurve = "linear" | "exponential" | "constant_product";
export type Decimal = string;
export type PoolType = "token" | "nft" | "trade";
export interface PoolInfo {
  asset_recipient?: Addr | null;
  bonding_curve: BondingCurve;
  collection: Addr;
  delta: Uint128;
  finders_fee_percent: Decimal;
  pool_type: PoolType;
  reinvest_nfts: boolean;
  reinvest_tokens: boolean;
  spot_price: Uint128;
  swap_fee_percent: Decimal;
}
export interface PoolNftSwap {
  nft_swaps: NftSwap[];
  pool_id: number;
}
export interface PoolQuoteResponse {
  pool_quotes: PoolQuote[];
}
export interface PoolQuote {
  collection: Addr;
  id: number;
  quote_price: Uint128;
}
export interface PoolsByIdResponse {
  pools: [number, Pool | null][];
}
export interface Pool {
  asset_recipient?: Addr | null;
  bonding_curve: BondingCurve;
  collection: Addr;
  delta: Uint128;
  finders_fee_percent: Decimal;
  id: number;
  is_active: boolean;
  nft_token_ids: string[];
  owner: Addr;
  pool_type: PoolType;
  reinvest_nfts: boolean;
  reinvest_tokens: boolean;
  spot_price: Uint128;
  swap_fee_percent: Decimal;
  total_tokens: Uint128;
}
export interface PoolsResponse {
  pools: Pool[];
}
export interface QueryOptionsForTupleOfUint128AndUint64 {
  descending?: boolean | null;
  limit?: number | null;
  start_after?: [Uint128, number] | null;
}
export interface QueryOptionsForUint64 {
  descending?: boolean | null;
  limit?: number | null;
  start_after?: number | null;
}
export type Timestamp = Uint64;
export type Uint64 = string;
export interface SwapParams {
  deadline: Timestamp;
  robust: boolean;
}
export type TransactionType = "sell" | "buy";
export interface SwapResponse {
  swaps: Swap[];
}
export interface Swap {
  finder_payment?: TokenPayment | null;
  network_fee: Uint128;
  nft_payment?: NftPayment | null;
  pool_id: number;
  royalty_payment?: TokenPayment | null;
  seller_payment?: TokenPayment | null;
  spot_price: Uint128;
  transaction_type: TransactionType;
}
export interface TokenPayment {
  address: string;
  amount: Uint128;
}
export interface NftPayment {
  address: string;
  nft_token_id: string;
}