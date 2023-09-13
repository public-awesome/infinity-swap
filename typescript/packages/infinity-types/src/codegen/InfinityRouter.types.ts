/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.30.1.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface InstantiateMsg {
  infinity_global: string;
}
export type ExecuteMsg = {
  swap_nfts_for_tokens: {
    collection: string;
    denom: string;
    filter_sources?: NftForTokensSource[] | null;
    sell_orders: SellOrder[];
    swap_params?: SwapParamsForString | null;
  };
} | {
  swap_tokens_for_nfts: {
    collection: string;
    denom: string;
    filter_sources?: TokensForNftSource[] | null;
    max_inputs: Uint128[];
    swap_params?: SwapParamsForString | null;
  };
};
export type NftForTokensSource = "infinity";
export type Uint128 = string;
export type TokensForNftSource = "infinity";
export interface SellOrder {
  input_token_id: string;
  min_output: Uint128;
}
export interface SwapParamsForString {
  asset_recipient?: string | null;
  robust?: boolean | null;
}
export type QueryMsg = {
  nfts_for_tokens: {
    collection: string;
    denom: string;
    filter_sources?: NftForTokensSource[] | null;
    limit: number;
  };
} | {
  tokens_for_nfts: {
    collection: string;
    denom: string;
    filter_sources?: TokensForNftSource[] | null;
    limit: number;
  };
};
export type Addr = string;
export type ArrayOfNftForTokensQuote = NftForTokensQuote[];
export interface NftForTokensQuote {
  address: Addr;
  amount: Uint128;
  source: NftForTokensSource;
}
export type ArrayOfTokensForNftQuote = TokensForNftQuote[];
export interface TokensForNftQuote {
  address: Addr;
  amount: Uint128;
  source: TokensForNftSource;
}