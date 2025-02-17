#![no_main]
#![no_std]

extern crate alloc;
use alloc::{boxed::Box, collections::BTreeSet, format, string::String, vec, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::{ContractHash, ContractPackageHash},
    runtime_args, ApiError, CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType,
    EntryPoints, Group, Key, Parameter, RuntimeArgs, URef, U256,
};
use casperlabs_contract_utils::{ContractContext, OnChainContractStorage};
use uniswap_v2_router::config::*;
use uniswap_v2_router::{self, UniswapV2Router};

#[derive(Default)]
struct Uniswap(OnChainContractStorage);

impl ContractContext<OnChainContractStorage> for Uniswap {
    fn storage(&self) -> &OnChainContractStorage {
        &self.0
    }
}

impl UniswapV2Router<OnChainContractStorage> for Uniswap {}

impl Uniswap {
    fn constructor(
        &mut self,
        factory: Key,
        wcspr: Key,
        library_hash: Key,
        contract_hash: ContractHash,
        package_hash: ContractPackageHash,
    ) {
        let _factory: ContractPackageHash =
            ContractPackageHash::from(factory.into_hash().unwrap_or_default());
        let _wcspr: ContractPackageHash =
            ContractPackageHash::from(wcspr.into_hash().unwrap_or_default());
        let _library_hash: ContractPackageHash =
            ContractPackageHash::from(library_hash.into_hash().unwrap_or_default());
        UniswapV2Router::init(
            self,
            _factory,
            _wcspr,
            _library_hash,
            Key::from(contract_hash),
            package_hash,
        );
    }
}

/// Constructor to initialize required key pairs
#[no_mangle]
fn constructor() {
    let factory: Key = runtime::get_named_arg("factory");
    let wcspr: Key = runtime::get_named_arg("wcspr");
    let library_hash: Key = runtime::get_named_arg("library_hash");
    let contract_hash: ContractHash = runtime::get_named_arg("contract_hash");
    let package_hash: ContractPackageHash = runtime::get_named_arg("package_hash");
    Uniswap::default().constructor(factory, wcspr, library_hash, contract_hash, package_hash);
}

#[no_mangle]
/// Add tokens to liquidity pool.
///
/// Parameters-> token_a:Key, token_b:Key, amount_a_desired:U256, amount_b_desired:U256, amount_a_min:U256, amount_b_min:U256, to:Key, deadline: U256, pair:Option<Key> , purse:URef
fn add_liquidity() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut1 as u16));
    }

    let token_a: Key = runtime::get_named_arg("token_a");
    let token_b: Key = runtime::get_named_arg("token_b");
    let amount_a_desired: U256 = runtime::get_named_arg("amount_a_desired");
    let amount_b_desired: U256 = runtime::get_named_arg("amount_b_desired");
    let amount_a_min: U256 = runtime::get_named_arg("amount_a_min");
    let amount_b_min: U256 = runtime::get_named_arg("amount_b_min");
    let to: Key = runtime::get_named_arg("to");
    let pair: Option<Key> = runtime::get_named_arg("pair");

    let _token_a = ContractPackageHash::from(token_a.into_hash().unwrap_or_default());
    let _token_b = ContractPackageHash::from(token_b.into_hash().unwrap_or_default());

    let (amount_a, amount_b, liquidity): (U256, U256, U256) = Uniswap::default().add_liquidity(
        _token_a,
        _token_b,
        amount_a_desired,
        amount_b_desired,
        amount_a_min,
        amount_b_min,
        to,
        pair,
    );
    runtime::ret(CLValue::from_t((amount_a, amount_b, liquidity)).unwrap_or_revert());
}

#[no_mangle]
/// Add tokens to liquidity pool.
///
/// Parameters-> token_a:Key, token_b:Key, amount_a_desired:U256, amount_b_desired:U256, amount_a_min:U256, amount_b_min:U256, to:Key, deadline: U256, pair:Option<Key> , purse:URef
fn add_liquidity_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut2 as u16));
    }

    let token_a: Key = runtime::get_named_arg("token_a");
    let token_b: Key = runtime::get_named_arg("token_b");
    let amount_a_desired: U256 = runtime::get_named_arg("amount_a_desired");
    let amount_b_desired: U256 = runtime::get_named_arg("amount_b_desired");
    let amount_a_min: U256 = runtime::get_named_arg("amount_a_min");
    let amount_b_min: U256 = runtime::get_named_arg("amount_b_min");
    let to: Key = runtime::get_named_arg("to");
    let pair: Option<Key> = runtime::get_named_arg("pair");

    let _token_a = ContractPackageHash::from(token_a.into_hash().unwrap_or_default());
    let _token_b = ContractPackageHash::from(token_b.into_hash().unwrap_or_default());

    let (_amount_a, _amount_b, _liquidity): (U256, U256, U256) = Uniswap::default().add_liquidity(
        _token_a,
        _token_b,
        amount_a_desired,
        amount_b_desired,
        amount_a_min,
        amount_b_min,
        to,
        pair,
    );
}

#[no_mangle]
/// Add cspr to liquidity pool.
///
/// Parameters-> token:Key, amount_token_desired:U256, amount_cspr_desired:U256, amount_token_min:U256, amount_cspr_min:U256, to:Key, deadline:U256, pair:Option<Key> , purse:URef
fn add_liquidity_cspr() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut3 as u16));
    }

    let token: Key = runtime::get_named_arg("token");
    let amount_token_desired: U256 = runtime::get_named_arg("amount_token_desired");
    let amount_cspr_desired: U256 = runtime::get_named_arg("amount_cspr_desired");
    let amount_token_min: U256 = runtime::get_named_arg("amount_token_min");
    let amount_cspr_min: U256 = runtime::get_named_arg("amount_cspr_min");
    let to: Key = runtime::get_named_arg("to");
    let pair: Option<Key> = runtime::get_named_arg("pair");
    let purse: URef = runtime::get_named_arg("purse");

    let _token = ContractPackageHash::from(token.into_hash().unwrap_or_default());
    let (amount_token, amount_cspr, liquidity): (U256, U256, U256) = Uniswap::default()
        .add_liquidity_cspr(
            _token,
            amount_token_desired,
            amount_cspr_desired,
            amount_token_min,
            amount_cspr_min,
            to,
            pair,
            purse,
        );
    runtime::ret(CLValue::from_t((amount_token, amount_cspr, liquidity)).unwrap_or_revert());
}

#[no_mangle]
/// Add cspr to liquidity pool.
///
/// Parameters-> token:Key, amount_token_desired:U256, amount_cspr_desired:U256, amount_token_min:U256, amount_cspr_min:U256, to:Key, deadline:U256, pair:Option<Key> , purse:URef
fn add_liquidity_cspr_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut4 as u16));
    }

    let token: Key = runtime::get_named_arg("token");
    let amount_token_desired: U256 = runtime::get_named_arg("amount_token_desired");
    let amount_cspr_desired: U256 = runtime::get_named_arg("amount_cspr_desired");
    let amount_token_min: U256 = runtime::get_named_arg("amount_token_min");
    let amount_cspr_min: U256 = runtime::get_named_arg("amount_cspr_min");
    let to: Key = runtime::get_named_arg("to");
    let pair: Option<Key> = runtime::get_named_arg("pair");
    let purse: URef = runtime::get_named_arg("purse");

    let _token = ContractPackageHash::from(token.into_hash().unwrap_or_default());
    let (_amount_token, _amount_cspr, _liquidity): (U256, U256, U256) = Uniswap::default()
        .add_liquidity_cspr(
            _token,
            amount_token_desired,
            amount_cspr_desired,
            amount_token_min,
            amount_cspr_min,
            to,
            pair,
            purse,
        );
}

#[no_mangle]
/// Remove from liquidity pool.
///
/// Parameters-> token_a:Key, token_b:Key, liquidity:U256, amount_a_min:U256, amount_b_min:U256, to:Key, deadline:U256
fn remove_liquidity() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut5 as u16));
    }

    let token_a: Key = runtime::get_named_arg("token_a");
    let token_b: Key = runtime::get_named_arg("token_b");
    let liquidity: U256 = runtime::get_named_arg("liquidity");
    let amount_a_min: U256 = runtime::get_named_arg("amount_a_min");
    let amount_b_min: U256 = runtime::get_named_arg("amount_b_min");
    let to: Key = runtime::get_named_arg("to");

    let _token_a = ContractPackageHash::from(token_a.into_hash().unwrap_or_default());
    let _token_b = ContractPackageHash::from(token_b.into_hash().unwrap_or_default());

    let (amount_a, amount_b): (U256, U256) = Uniswap::default().remove_liquidity(
        _token_a,
        _token_b,
        liquidity,
        amount_a_min,
        amount_b_min,
        to,
    );
    runtime::ret(CLValue::from_t((amount_a, amount_b)).unwrap_or_revert());
}

#[no_mangle]
/// Remove from liquidity pool.
///
/// Parameters-> token_a:Key, token_b:Key, liquidity:U256, amount_a_min:U256, amount_b_min:U256, to:Key, deadline:U256
fn remove_liquidity_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut6 as u16));
    }

    let token_a: Key = runtime::get_named_arg("token_a");
    let token_b: Key = runtime::get_named_arg("token_b");
    let liquidity: U256 = runtime::get_named_arg("liquidity");
    let amount_a_min: U256 = runtime::get_named_arg("amount_a_min");
    let amount_b_min: U256 = runtime::get_named_arg("amount_b_min");
    let to: Key = runtime::get_named_arg("to");

    let _token_a = ContractPackageHash::from(token_a.into_hash().unwrap_or_default());
    let _token_b = ContractPackageHash::from(token_b.into_hash().unwrap_or_default());

    let (_amount_a, _amount_b): (U256, U256) = Uniswap::default().remove_liquidity(
        _token_a,
        _token_b,
        liquidity,
        amount_a_min,
        amount_b_min,
        to,
    );
}

#[no_mangle]
/// Remove cspr from liquidity pool.
///
/// Parameters-> token:Key, liquidity:U256, amount_token_min:U256, amount_cspr_min:U256, to:Key, deadline:U256
fn remove_liquidity_cspr() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut7 as u16));
    }

    let token: Key = runtime::get_named_arg("token");
    let liquidity: U256 = runtime::get_named_arg("liquidity");
    let amount_token_min: U256 = runtime::get_named_arg("amount_token_min");
    let amount_cspr_min: U256 = runtime::get_named_arg("amount_cspr_min");
    let to: Key = runtime::get_named_arg("to");
    let to_purse: URef = runtime::get_named_arg("to_purse");

    let _token = ContractPackageHash::from(token.into_hash().unwrap_or_default());
    let (amount_token, amount_cspr): (U256, U256) = Uniswap::default().remove_liquidity_cspr(
        _token,
        liquidity,
        amount_token_min,
        amount_cspr_min,
        to,
        to_purse,
    );
    runtime::ret(CLValue::from_t((amount_token, amount_cspr)).unwrap_or_revert());
}

#[no_mangle]
/// Remove cspr from liquidity pool.
///
/// Parameters-> token:Key, liquidity:U256, amount_token_min:U256, amount_cspr_min:U256, to:Key, deadline:U256
fn remove_liquidity_cspr_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut8 as u16));
    }

    let token: Key = runtime::get_named_arg("token");
    let liquidity: U256 = runtime::get_named_arg("liquidity");
    let amount_token_min: U256 = runtime::get_named_arg("amount_token_min");
    let amount_cspr_min: U256 = runtime::get_named_arg("amount_cspr_min");
    let to: Key = runtime::get_named_arg("to");
    let to_purse: URef = runtime::get_named_arg("to_purse");

    let _token = ContractPackageHash::from(token.into_hash().unwrap_or_default());
    let (_amount_token, _amount_cspr): (U256, U256) = Uniswap::default().remove_liquidity_cspr(
        _token,
        liquidity,
        amount_token_min,
        amount_cspr_min,
        to,
        to_purse,
    );
}

#[no_mangle]
/// Remove from liquidity pool with permit.
///
/// Parameters-> token_a:Key, token_b:Key, liquidity:U256, amount_a_min:U256, amount_b_min:U256, to:Key, approve_max:bool
/// public_key:String, signature: String, deadline:U256,  to_purse: URef
fn remove_liquidity_with_permit() {
    let token_a: Key = runtime::get_named_arg("token_a");
    let token_b: Key = runtime::get_named_arg("token_b");
    let liquidity: U256 = runtime::get_named_arg("liquidity");
    let amount_a_min: U256 = runtime::get_named_arg("amount_a_min");
    let amount_b_min: U256 = runtime::get_named_arg("amount_b_min");
    let to: Key = runtime::get_named_arg("to");
    let approve_max: bool = runtime::get_named_arg("approve_max");
    let public_key: String = runtime::get_named_arg("public_key");
    let signature: String = runtime::get_named_arg("signature");
    let deadline: U256 = runtime::get_named_arg("deadline");

    let _token_a = ContractPackageHash::from(token_a.into_hash().unwrap_or_default());
    let _token_b = ContractPackageHash::from(token_b.into_hash().unwrap_or_default());

    let (amount_a, amount_b): (U256, U256) = Uniswap::default().remove_liquidity_with_permit(
        _token_a,
        _token_b,
        liquidity,
        amount_a_min,
        amount_b_min,
        to,
        approve_max,
        public_key,
        signature,
        deadline,
    );
    runtime::ret(CLValue::from_t((amount_a, amount_b)).unwrap_or_revert());
}

#[no_mangle]
/// Remove from liquidity pool with permit.
///
/// Parameters-> token_a:Key, token_b:Key, liquidity:U256, amount_a_min:U256, amount_b_min:U256, to:Key, approve_max:bool
/// public_key:String, signature: String, deadline:U256
fn remove_liquidity_with_permit_js_client() {
    let token_a: Key = runtime::get_named_arg("token_a");
    let token_b: Key = runtime::get_named_arg("token_b");
    let liquidity: U256 = runtime::get_named_arg("liquidity");
    let amount_a_min: U256 = runtime::get_named_arg("amount_a_min");
    let amount_b_min: U256 = runtime::get_named_arg("amount_b_min");
    let to: Key = runtime::get_named_arg("to");
    let approve_max: bool = runtime::get_named_arg("approve_max");
    let public_key: String = runtime::get_named_arg("public_key");
    let signature: String = runtime::get_named_arg("signature");
    let deadline: U256 = runtime::get_named_arg("deadline");

    let _token_a = ContractPackageHash::from(token_a.into_hash().unwrap_or_default());
    let _token_b = ContractPackageHash::from(token_b.into_hash().unwrap_or_default());

    let (_amount_a, _amount_b): (U256, U256) = Uniswap::default().remove_liquidity_with_permit(
        _token_a,
        _token_b,
        liquidity,
        amount_a_min,
        amount_b_min,
        to,
        approve_max,
        public_key,
        signature,
        deadline,
    );
}

#[no_mangle]
/// Remove cspr from liquidity pool with permit.
///
/// Parameters-> token:ContractPackageHash, liquidity:U256, amount_token_min:U256, amount_cspr_min:U256, to:Key, approve_max:bool,
/// deadline:U256, public_key:String, signature: String, to_purse: URef

fn remove_liquidity_cspr_with_permit() {
    let token: Key = runtime::get_named_arg("token");
    let liquidity: U256 = runtime::get_named_arg("liquidity");
    let amount_token_min: U256 = runtime::get_named_arg("amount_token_min");
    let amount_cspr_min: U256 = runtime::get_named_arg("amount_cspr_min");
    let to: Key = runtime::get_named_arg("to");
    let approve_max: bool = runtime::get_named_arg("approve_max");
    let public_key: String = runtime::get_named_arg("public_key");
    let signature: String = runtime::get_named_arg("signature");
    let deadline: U256 = runtime::get_named_arg("deadline");
    let to_purse: URef = runtime::get_named_arg("to_purse");

    let _token = ContractPackageHash::from(token.into_hash().unwrap_or_default());
    let (amount_token, amount_cspr): (U256, U256) = Uniswap::default()
        .remove_liquidity_cspr_with_permit(
            _token,
            liquidity,
            amount_token_min,
            amount_cspr_min,
            to,
            approve_max,
            public_key,
            signature,
            deadline,
            to_purse,
        );
    runtime::ret(CLValue::from_t((amount_token, amount_cspr)).unwrap_or_revert());
}

#[no_mangle]
/// Remove cspr from liquidity pool with permit.
///
/// Parameters-> token:ContractPackageHash, liquidity:U256, amount_token_min:U256, amount_cspr_min:U256, to:Key, approve_max:bool,
/// deadline:U256, public_key:String, signature: String

fn remove_liquidity_cspr_with_permit_js_client() {
    let token: Key = runtime::get_named_arg("token");
    let liquidity: U256 = runtime::get_named_arg("liquidity");
    let amount_token_min: U256 = runtime::get_named_arg("amount_token_min");
    let amount_cspr_min: U256 = runtime::get_named_arg("amount_cspr_min");
    let to: Key = runtime::get_named_arg("to");
    let approve_max: bool = runtime::get_named_arg("approve_max");
    let public_key: String = runtime::get_named_arg("public_key");
    let signature: String = runtime::get_named_arg("signature");
    let deadline: U256 = runtime::get_named_arg("deadline");
    let to_purse: URef = runtime::get_named_arg("to_purse");

    let _token = ContractPackageHash::from(token.into_hash().unwrap_or_default());
    let (_amount_token, _amount_cspr): (U256, U256) = Uniswap::default()
        .remove_liquidity_cspr_with_permit(
            _token,
            liquidity,
            amount_token_min,
            amount_cspr_min,
            to,
            approve_max,
            public_key,
            signature,
            deadline,
            to_purse,
        );
}

#[no_mangle]
/// Swap exact tokens for tokens.
///
/// Parameters-> amount_in:U256, amount_out_min:U256, path:Vec<Key>, to:Key, deadline:U256
fn swap_exact_tokens_for_tokens() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut9 as u16));
    }

    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let amount_out_min: U256 = runtime::get_named_arg("amount_out_min");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: Key = runtime::get_named_arg("to");

    let amounts: Vec<U256> =
        Uniswap::default().swap_exact_tokens_for_tokens(amount_in, amount_out_min, path, to);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert());
}

#[no_mangle]
/// Swap exact tokens for tokens.
///
/// Parameters-> amount_in:U256, amount_out_min:U256, path:Vec<Key>, to:Key, deadline:U256
fn swap_exact_tokens_for_tokens_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut10 as u16));
    }

    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let amount_out_min: U256 = runtime::get_named_arg("amount_out_min");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: Key = runtime::get_named_arg("to");

    let _amounts: Vec<U256> =
        Uniswap::default().swap_exact_tokens_for_tokens(amount_in, amount_out_min, path, to);
}

#[no_mangle]
/// Swap tokens for exact tokens.
///
/// Parameters-> amount_out:U256, amount_in_max:U256, path:Vec<Key>, to:Key, deadline:U256
fn swap_tokens_for_exact_tokens() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut11 as u16));
    }

    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let amount_in_max: U256 = runtime::get_named_arg("amount_in_max");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: Key = runtime::get_named_arg("to");

    let amounts: Vec<U256> =
        Uniswap::default().swap_tokens_for_exact_tokens(amount_out, amount_in_max, path, to);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert());
}

#[no_mangle]
/// Swap tokens for exact tokens.
///
/// Parameters-> amount_out:U256, amount_in_max:U256, path:Vec<Key>, to:Key, deadline:U256
fn swap_tokens_for_exact_tokens_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut12 as u16));
    }

    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let amount_in_max: U256 = runtime::get_named_arg("amount_in_max");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: Key = runtime::get_named_arg("to");

    let _amounts: Vec<U256> =
        Uniswap::default().swap_tokens_for_exact_tokens(amount_out, amount_in_max, path, to);
}

#[no_mangle]
/// Swap exact cspr for tokens.
///
/// Parameters-> amount_out_min:U256, amount_in:U256, path:Vec<Key>, to:Key, deadline:U256, purse:URef
fn swap_exact_cspr_for_tokens() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut13 as u16));
    }

    let amount_out_min: U256 = runtime::get_named_arg("amount_out_min");
    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: Key = runtime::get_named_arg("to");
    let purse: URef = runtime::get_named_arg("purse");

    let amounts: Vec<U256> =
        Uniswap::default().swap_exact_cspr_for_tokens(amount_out_min, amount_in, path, to, purse);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert());
}

#[no_mangle]
/// Swap exact cspr for tokens.
///
/// Parameters-> amount_out_min:U256, amount_in:U256, path:Vec<Key>, to:Key, deadline:U256, purse:URef
fn swap_exact_cspr_for_tokens_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut14 as u16));
    }

    let amount_out_min: U256 = runtime::get_named_arg("amount_out_min");
    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: Key = runtime::get_named_arg("to");
    let purse: URef = runtime::get_named_arg("purse");

    let _amounts: Vec<U256> =
        Uniswap::default().swap_exact_cspr_for_tokens(amount_out_min, amount_in, path, to, purse);
}

#[no_mangle]
/// Swap tokens for exact cspr.
///
/// Parameters-> amount_out:U256, amount_in_max:U256, path:Vec<Key>, to:Key, deadline:U256
fn swap_tokens_for_exact_cspr() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut15 as u16));
    }

    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let amount_in_max: U256 = runtime::get_named_arg("amount_in_max");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: URef = runtime::get_named_arg("to");

    let amounts: Vec<U256> =
        Uniswap::default().swap_tokens_for_exact_cspr(amount_out, amount_in_max, path, to);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert());
}

#[no_mangle]
/// Swap tokens for exact cspr.
///
/// Parameters-> amount_out:U256, amount_in_max:U256, path:Vec<Key>, to:Key, deadline:U256
fn swap_tokens_for_exact_cspr_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut16 as u16));
    }

    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let amount_in_max: U256 = runtime::get_named_arg("amount_in_max");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: URef = runtime::get_named_arg("to");

    let _amounts: Vec<U256> =
        Uniswap::default().swap_tokens_for_exact_cspr(amount_out, amount_in_max, path, to);
}

#[no_mangle]
/// Swap exact tokens for cspr.
///
/// Parameters-> amount_in:U256, amount_out_min:U256, path:Vec<Key>, to:Key, deadline:U256
fn swap_exact_tokens_for_cspr() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut17 as u16));
    }

    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let amount_out_min: U256 = runtime::get_named_arg("amount_out_min");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: URef = runtime::get_named_arg("to");

    let amounts: Vec<U256> =
        Uniswap::default().swap_exact_tokens_for_cspr(amount_in, amount_out_min, path, to);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert());
}

#[no_mangle]
/// Swap exact tokens for cspr.
///
/// Parameters-> amount_in:U256, amount_out_min:U256, path:Vec<Key>, to:Key, deadline:U256
fn swap_exact_tokens_for_cspr_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut18 as u16));
    }

    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let amount_out_min: U256 = runtime::get_named_arg("amount_out_min");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: URef = runtime::get_named_arg("to");

    let _amounts: Vec<U256> =
        Uniswap::default().swap_exact_tokens_for_cspr(amount_in, amount_out_min, path, to);
}

/// Swap cspr for exact tokens
///
/// Parameters-> amount_out:U256, amount_in_max:U256, path:Vec<Key>, to:Key, deadline:U256, purse:URef

#[no_mangle]
fn swap_cspr_for_exact_tokens() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut19 as u16));
    }

    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let amount_in_max: U256 = runtime::get_named_arg("amount_in_max");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: Key = runtime::get_named_arg("to");
    let purse: URef = runtime::get_named_arg("purse");

    let amounts: Vec<U256> =
        Uniswap::default().swap_cspr_for_exact_tokens(amount_out, amount_in_max, path, to, purse);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert());
}

#[no_mangle]
fn swap_cspr_for_exact_tokens_js_client() {
    let deadline: U256 = runtime::get_named_arg("deadline");
    if !(Uniswap::default().ensure(deadline)) {
        runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterTimedOut20 as u16));
    }

    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let amount_in_max: U256 = runtime::get_named_arg("amount_in_max");
    let path: Vec<String> = runtime::get_named_arg("path");
    let to: Key = runtime::get_named_arg("to");
    let purse: URef = runtime::get_named_arg("purse");

    let _amounts: Vec<U256> =
        Uniswap::default().swap_cspr_for_exact_tokens(amount_out, amount_in_max, path, to, purse);
}

#[no_mangle]
// given some amount of an asset and pair reserves, returns an equivalent amount of the other asset
fn quote() {
    let amount_a: U256 = runtime::get_named_arg("amount_a");
    let reserve_a: U256 = runtime::get_named_arg("reserve_a");
    let reserve_b: U256 = runtime::get_named_arg("reserve_b");

    let amount_b: U256 = Uniswap::quote(amount_a, reserve_a, reserve_b);
    runtime::ret(CLValue::from_t(amount_b).unwrap_or_revert())
}

#[no_mangle]
// given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
fn get_amount_out() {
    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let reserve_in: U256 = runtime::get_named_arg("reserve_in");
    let reserve_out: U256 = runtime::get_named_arg("reserve_out");

    let amount_out: U256 = Uniswap::get_amount_out(amount_in, reserve_in, reserve_out);
    runtime::ret(CLValue::from_t(amount_out).unwrap_or_revert())
}

#[no_mangle]
// given an output amount of an asset and pair reserves, returns a required input amount of the other asset
fn get_amount_in() {
    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let reserve_in: U256 = runtime::get_named_arg("reserve_in");
    let reserve_out: U256 = runtime::get_named_arg("reserve_out");

    let amount_in: U256 = Uniswap::get_amount_in(amount_out, reserve_in, reserve_out);
    runtime::ret(CLValue::from_t(amount_in).unwrap_or_revert())
}

#[no_mangle]
// performs chained getAmountOut calculations on any number of pairs
fn get_amounts_out() {
    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let path: Vec<Key> = runtime::get_named_arg("path");

    let amounts: Vec<U256> = Uniswap::get_amounts_out(amount_in, path);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert())
}

#[no_mangle]
// performs chained getAmountIn calculations on any number of pairs
fn get_amounts_in() {
    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let path: Vec<Key> = runtime::get_named_arg("path");

    let amounts: Vec<U256> = Uniswap::get_amounts_in(amount_out, path);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert())
}

#[no_mangle]
fn package_hash() {
    let ret: ContractPackageHash = Uniswap::default().get_package_hash();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        "constructor",
        vec![
            Parameter::new("factory", CLType::Key),
            Parameter::new("wcspr", CLType::Key),
            Parameter::new("library_hash", CLType::Key),
            Parameter::new("contract_hash", ContractHash::cl_type()),
            Parameter::new("package_hash", ContractPackageHash::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Groups(vec![Group::new("constructor")]),
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("add_liquidity"),
        vec![
            Parameter::new("token_a", CLType::Key),
            Parameter::new("token_b", CLType::Key),
            Parameter::new("amount_a_desired", CLType::U256),
            Parameter::new("amount_b_desired", CLType::U256),
            Parameter::new("amount_a_min", CLType::U256),
            Parameter::new("amount_b_min", CLType::U256),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("pair", CLType::Option(Box::new(CLType::Key))),
        ],
        CLType::Tuple3([
            Box::new(CLType::U256),
            Box::new(CLType::U256),
            Box::new(CLType::U256),
        ]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("add_liquidity_js_client"),
        vec![
            Parameter::new("token_a", CLType::Key),
            Parameter::new("token_b", CLType::Key),
            Parameter::new("amount_a_desired", CLType::U256),
            Parameter::new("amount_b_desired", CLType::U256),
            Parameter::new("amount_a_min", CLType::U256),
            Parameter::new("amount_b_min", CLType::U256),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("pair", CLType::Option(Box::new(CLType::Key))),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("add_liquidity_cspr"),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("amount_token_desired", CLType::U256),
            Parameter::new("amount_cspr_desired", CLType::U256),
            Parameter::new("amount_token_min", CLType::U256),
            Parameter::new("amount_cspr_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("pair", CLType::Option(Box::new(CLType::Key))),
            Parameter::new("purse", CLType::URef),
        ],
        CLType::Tuple3([
            Box::new(CLType::U256),
            Box::new(CLType::U256),
            Box::new(CLType::U256),
        ]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("add_liquidity_cspr_js_client"),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("amount_token_desired", CLType::U256),
            Parameter::new("amount_cspr_desired", CLType::U256),
            Parameter::new("amount_token_min", CLType::U256),
            Parameter::new("amount_cspr_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("pair", CLType::Option(Box::new(CLType::Key))),
            Parameter::new("purse", CLType::URef),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("remove_liquidity"),
        vec![
            Parameter::new("token_a", Key::cl_type()),
            Parameter::new("token_b", Key::cl_type()),
            Parameter::new("liquidity", CLType::U256),
            Parameter::new("amount_a_min", CLType::U256),
            Parameter::new("amount_b_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
        ],
        CLType::Tuple2([Box::new(CLType::U256), Box::new(CLType::U256)]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("remove_liquidity_js_client"),
        vec![
            Parameter::new("token_a", Key::cl_type()),
            Parameter::new("token_b", Key::cl_type()),
            Parameter::new("liquidity", CLType::U256),
            Parameter::new("amount_a_min", CLType::U256),
            Parameter::new("amount_b_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("remove_liquidity_cspr"),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("liquidity", CLType::U256),
            Parameter::new("amount_token_min", CLType::U256),
            Parameter::new("amount_cspr_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("to_purse", CLType::URef),
        ],
        CLType::Tuple2([Box::new(CLType::U256), Box::new(CLType::U256)]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("remove_liquidity_cspr_js_client"),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("liquidity", CLType::U256),
            Parameter::new("amount_token_min", CLType::U256),
            Parameter::new("amount_cspr_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("to_purse", CLType::URef),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("remove_liquidity_with_permit"),
        vec![
            Parameter::new("token_a", Key::cl_type()),
            Parameter::new("token_b", Key::cl_type()),
            Parameter::new("liquidity", CLType::U256),
            Parameter::new("amount_a_min", CLType::U256),
            Parameter::new("amount_b_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("approve_max", CLType::Bool),
            Parameter::new("public_key", CLType::String),
            Parameter::new("signature", CLType::String),
        ],
        CLType::Tuple2([Box::new(CLType::U256), Box::new(CLType::U256)]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        String::from("remove_liquidity_with_permit_js_client"),
        vec![
            Parameter::new("token_a", Key::cl_type()),
            Parameter::new("token_b", Key::cl_type()),
            Parameter::new("liquidity", CLType::U256),
            Parameter::new("amount_a_min", CLType::U256),
            Parameter::new("amount_b_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("approve_max", CLType::Bool),
            Parameter::new("public_key", CLType::String),
            Parameter::new("signature", CLType::String),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("remove_liquidity_cspr_with_permit"),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("liquidity", CLType::U256),
            Parameter::new("amount_token_min", CLType::U256),
            Parameter::new("amount_cspr_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("approve_max", CLType::Bool),
            Parameter::new("public_key", CLType::String),
            Parameter::new("signature", CLType::String),
            Parameter::new("to_purse", CLType::URef),
        ],
        CLType::Tuple2([Box::new(CLType::U256), Box::new(CLType::U256)]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("remove_liquidity_cspr_with_permit_js_client"),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("liquidity", CLType::U256),
            Parameter::new("amount_token_min", CLType::U256),
            Parameter::new("amount_cspr_min", CLType::U256),
            Parameter::new("to", Key::cl_type()),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("approve_max", CLType::Bool),
            Parameter::new("public_key", CLType::String),
            Parameter::new("signature", CLType::String),
            Parameter::new("to_purse", CLType::URef),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_exact_tokens_for_tokens"),
        vec![
            Parameter::new("amount_in", CLType::U256),
            Parameter::new("amount_out_min", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
        ],
        CLType::List(Box::new(CLType::U256)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_exact_tokens_for_tokens_js_client"),
        vec![
            Parameter::new("amount_in", CLType::U256),
            Parameter::new("amount_out_min", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_tokens_for_exact_tokens"),
        vec![
            Parameter::new("amount_out", CLType::U256),
            Parameter::new("amount_in_max", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
        ],
        CLType::List(Box::new(CLType::U256)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_tokens_for_exact_tokens_js_client"),
        vec![
            Parameter::new("amount_out", CLType::U256),
            Parameter::new("amount_in_max", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_exact_cspr_for_tokens"),
        vec![
            Parameter::new("amount_out_min", CLType::U256),
            Parameter::new("amount_in", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("purse", CLType::URef),
        ],
        CLType::List(Box::new(CLType::U256)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_exact_cspr_for_tokens_js_client"),
        vec![
            Parameter::new("amount_out_min", CLType::U256),
            Parameter::new("amount_in", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("purse", CLType::URef),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_tokens_for_exact_cspr"),
        vec![
            Parameter::new("amount_out", CLType::U256),
            Parameter::new("amount_in_max", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::URef),
            Parameter::new("deadline", CLType::U256),
        ],
        CLType::List(Box::new(CLType::U256)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_tokens_for_exact_cspr_js_client"),
        vec![
            Parameter::new("amount_out", CLType::U256),
            Parameter::new("amount_in_max", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::URef),
            Parameter::new("deadline", CLType::U256),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_exact_tokens_for_cspr"),
        vec![
            Parameter::new("amount_in", CLType::U256),
            Parameter::new("amount_out_min", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::URef), // purse to transfer cspr to
            Parameter::new("deadline", CLType::U256),
        ],
        CLType::List(Box::new(CLType::U256)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_exact_tokens_for_cspr_js_client"),
        vec![
            Parameter::new("amount_in", CLType::U256),
            Parameter::new("amount_out_min", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::URef), // purse to transfer cspr to
            Parameter::new("deadline", CLType::U256),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_cspr_for_exact_tokens"),
        vec![
            Parameter::new("amount_out", CLType::U256),
            Parameter::new("amount_in_max", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("purse", CLType::URef),
        ],
        CLType::List(Box::new(CLType::U256)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("swap_cspr_for_exact_tokens_js_client"),
        vec![
            Parameter::new("amount_out", CLType::U256),
            Parameter::new("amount_in_max", CLType::U256),
            Parameter::new("path", CLType::List(Box::new(String::cl_type()))),
            Parameter::new("to", CLType::Key),
            Parameter::new("deadline", CLType::U256),
            Parameter::new("purse", CLType::URef),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "quote",
        vec![
            Parameter::new("amount_a", U256::cl_type()),
            Parameter::new("reserve_a", U256::cl_type()),
            Parameter::new("reserve_b", U256::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("get_amount_out"),
        vec![
            Parameter::new("amount_in", U256::cl_type()),
            Parameter::new("reserve_in", U256::cl_type()),
            Parameter::new("reserve_out", U256::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("get_amount_in"),
        vec![
            Parameter::new("amount_out", U256::cl_type()),
            Parameter::new("reserve_in", U256::cl_type()),
            Parameter::new("reserve_out", U256::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("get_amounts_out"),
        vec![
            Parameter::new("amount_in", U256::cl_type()),
            Parameter::new("path", CLType::List(Box::new(Key::cl_type()))),
        ],
        CLType::List(Box::new(U256::cl_type())),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("get_amounts_in"),
        vec![
            Parameter::new("amount_out", U256::cl_type()),
            Parameter::new("path", CLType::List(Box::new(Key::cl_type()))),
        ],
        CLType::List(Box::new(U256::cl_type())),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "package_hash",
        vec![],
        ContractPackageHash::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points
}

// All session code must have a `call` entrypoint.
#[no_mangle]
fn call() {
    // Store contract in the account's named keys. Contract name must be same for all new versions of the contracts
    let contract_name: alloc::string::String = runtime::get_named_arg("contract_name");

    // If this is the first deployment
    if !runtime::has_key(&format!("{}_package_hash", contract_name)) {
        // Build new package with initial a first version of the contract.
        let (package_hash, access_token) = storage::create_contract_package_at_hash();
        let (contract_hash, _): (ContractHash, _) =
            storage::add_contract_version(package_hash, get_entry_points(), Default::default());

        let factory: Key = runtime::get_named_arg("factory");
        let wcspr: Key = runtime::get_named_arg("wcspr");
        let library_hash: Key = runtime::get_named_arg("library");

        // Prepare constructor args
        let constructor_args = runtime_args! {
            "factory" => factory,
            "wcspr" => wcspr,
            "library_hash" =>  library_hash,
            "contract_hash" => contract_hash,
            "package_hash" => package_hash,
        };

        // Add the constructor group to the package hash with a single URef.
        let constructor_access: URef =
            storage::create_contract_user_group(package_hash, "constructor", 1, Default::default())
                .unwrap_or_revert()
                .pop()
                .unwrap_or_revert();

        // Call the constructor entry point
        let _: () =
            runtime::call_versioned_contract(package_hash, None, "constructor", constructor_args);

        // Remove all URefs from the constructor group, so no one can call it for the second time.
        let mut urefs = BTreeSet::new();
        urefs.insert(constructor_access);
        storage::remove_contract_user_group_urefs(package_hash, "constructor", urefs)
            .unwrap_or_revert();

        runtime::put_key(
            &format!("{}_package_hash", contract_name),
            package_hash.into(),
        );
        runtime::put_key(
            &format!("{}_package_hash_wrapped", contract_name),
            storage::new_uref(package_hash).into(),
        );
        runtime::put_key(
            &format!("{}_contract_hash", contract_name),
            contract_hash.into(),
        );
        runtime::put_key(
            &format!("{}_contract_hash_wrapped", contract_name),
            storage::new_uref(contract_hash).into(),
        );
        runtime::put_key(
            &format!("{}_package_access_token", contract_name),
            access_token.into(),
        );
    } else {
        // this is a contract upgrade

        let package_hash: ContractPackageHash =
            runtime::get_key(&format!("{}_package_hash", contract_name))
                .unwrap_or_revert()
                .into_hash()
                .unwrap()
                .into();

        let (contract_hash, _): (ContractHash, _) =
            storage::add_contract_version(package_hash, get_entry_points(), Default::default());

        // update contract hash
        runtime::put_key(
            &format!("{}_contract_hash", contract_name),
            contract_hash.into(),
        );
        runtime::put_key(
            &format!("{}_contract_hash_wrapped", contract_name),
            storage::new_uref(contract_hash).into(),
        );
    }
}
