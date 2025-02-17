extern crate alloc;
use alloc::{string::String, vec::Vec};

use casper_contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::FromBytes, contracts::ContractPackageHash, runtime_args, ApiError, BlockTime,
    CLTyped, Key, RuntimeArgs, URef, U128, U256, U512,
};
use casperlabs_contract_utils::{ContractContext, ContractStorage};

use crate::alloc::string::ToString;
use crate::config::*;
use crate::data::{self};
use crate::transfer_helper::transfer_helper;
use alloc::collections::BTreeMap;
use casper_contract::contract_api::storage;

pub enum ROUTEREvent {
    AddReserves {
        user: Key,
        reserve0: U256,
        reserve1: U256,
        pair_contract_hash: ContractPackageHash,
    },
    RemoveReserves {
        user: Key,
        reserve0: U256,
        reserve1: U256,
        pair_contract_hash: ContractPackageHash,
    },
}

impl ROUTEREvent {
    pub fn type_name(&self) -> String {
        match self {
            ROUTEREvent::AddReserves {
                user: _,
                reserve0: _,
                reserve1: _,
                pair_contract_hash: _,
            } => "addreserves",
            ROUTEREvent::RemoveReserves {
                user: _,
                reserve0: _,
                reserve1: _,
                pair_contract_hash: _,
            } => "removereserves",
        }
        .to_string()
    }
}

pub trait UniswapV2Router<Storage: ContractStorage>: ContractContext<Storage> {
    // Will be called by constructor
    fn init(
        &mut self,
        factory: ContractPackageHash,
        wcspr: ContractPackageHash,
        library_hash: ContractPackageHash,
        contract_hash: Key,
        package_hash: ContractPackageHash,
    ) {
        data::set_factory(factory);
        data::set_wcspr(wcspr);
        data::set_library_hash(library_hash);
        data::set_self_hash(contract_hash);
        data::set_package_hash(package_hash);
    }

    fn add_liquidity(
        &mut self,
        token_a: ContractPackageHash,
        token_b: ContractPackageHash,
        amount_a_desired: U256,
        amount_b_desired: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        to: Key,
        pair: Option<Key>,
    ) -> (U256, U256, U256) {
        let factory: ContractPackageHash = data::factory();

        if amount_a_desired <= 0.into() {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterAmountADesiredIsZero as u16,
            ));
        }
        if amount_b_desired <= 0.into() {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterAmountBDesiredIsZero as u16,
            ));
        }

        let (amount_a, amount_b): (U256, U256) = Self::_add_liquidity(
            token_a,
            token_b,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
            pair,
        );

        // // call pair_for from library contract
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => Key::from(token_a),
            "token_b" => Key::from(token_b)
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );
        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let pair_package_hash: ContractPackageHash =
            runtime::call_versioned_contract(pair, None, "package_hash", runtime_args! {});

        let result: Result<(), u32> = transfer_helper::safe_transfer_from(
            Key::from(token_a),
            self.get_caller(),
            Key::from(Key::from(pair_package_hash)),
            amount_a,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed1 as u16,
            ));
        }

        let result: Result<(), u32> = transfer_helper::safe_transfer_from(
            Key::from(token_b),
            self.get_caller(),
            Key::from(Key::from(pair_package_hash)),
            amount_b,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed2 as u16,
            ));
        }

        // call mint function from IUniswapV2Pair contract
        let args: RuntimeArgs = runtime_args! {
            "to" => to,
        };

        let liquidity: U256 = Self::call_versioned_contract(
            &pair.to_formatted_string(),
            uniswapv2_contract_methods::PAIR_MINT,
            args,
        );
        self.emit(&ROUTEREvent::AddReserves {
            user: to,
            reserve0: amount_a,
            reserve1: amount_b,
            pair_contract_hash: pair,
        });
        (amount_a, amount_b, liquidity)
    }

    fn add_liquidity_cspr(
        &mut self,
        token: ContractPackageHash,
        amount_token_desired: U256,
        amount_cspr_desired: U256,
        amount_token_min: U256,
        amount_cspr_min: U256,
        to: Key,
        pair: Option<Key>,
        caller_purse: URef,
    ) -> (U256, U256, U256) {
        let wcspr: ContractPackageHash = data::wcspr();
        let factory: ContractPackageHash = data::factory();

        let (amount_token, amount_cspr): (U256, U256) = Self::_add_liquidity(
            token,
            wcspr,
            amount_token_desired,
            amount_cspr_desired,
            amount_token_min,
            amount_cspr_min,
            pair,
        );

        // // call pair_for from library contract
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => Key::from(token),
            "token_b" => Key::from(wcspr)
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );

        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let pair_package_hash: ContractPackageHash =
            runtime::call_versioned_contract(pair, None, "package_hash", runtime_args! {});

        if amount_token <= 0.into() {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterAmountTokenIsZero as u16,
            ));
        }

        // call safe_transfer_from from TransferHelper
        let result: Result<(), u32> = transfer_helper::safe_transfer_from(
            Key::from(token),
            self.get_caller(),
            Key::from(pair_package_hash),
            amount_token,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed3 as u16,
            ));
        }

        let self_purse = system::create_purse(); // create new temporary purse and transfer cspr from caller purse to this
        let _: () = system::transfer_from_purse_to_purse(
            caller_purse,
            self_purse,
            U512::from(amount_cspr.as_u128()),
            None,
        )
        .unwrap_or_revert();

        // this call will submit cspr to the wcspr contract and in return get wcspr tokens which will be sent to pair

        let args: RuntimeArgs = runtime_args! {
            "amount" => U512::from(amount_cspr.as_u128()),
            "purse" => self_purse
        };
        let result: Result<(), u32> = Self::call_versioned_contract(
            &wcspr.to_formatted_string(),
            uniswapv2_contract_methods::WCSPR_DEPOSIT,
            args,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed4 as u16,
            ));
        }
        // call transfer method from wcspr
        let args: RuntimeArgs = runtime_args! {
            "recipient" => Key::from(pair_package_hash),
            "amount" => amount_cspr
        };

        let result: Result<(), u32> = Self::call_versioned_contract(
            &wcspr.to_formatted_string(),
            uniswapv2_contract_methods::WCSPR_TRANSFER,
            args,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed5 as u16,
            ));
        }
        // call mint function from pair contract
        let args: RuntimeArgs = runtime_args! {
            "to" => to,
        };

        let liquidity: U256 = Self::call_versioned_contract(
            &pair.to_formatted_string(),
            uniswapv2_contract_methods::PAIR_MINT,
            args,
        );
        self.emit(&ROUTEREvent::AddReserves {
            user: to,
            reserve0: amount_token,
            reserve1: amount_cspr,
            pair_contract_hash: pair,
        });
        // No need to transfer the leftover cspr, because we are already taking the exact amount out from the caller purse
        (amount_token, amount_cspr, liquidity)
    }

    fn remove_liquidity(
        &mut self,
        token_a: ContractPackageHash,
        token_b: ContractPackageHash,
        liquidity: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        to: Key,
    ) -> (U256, U256) {
        let factory: ContractPackageHash = data::factory();

        // call pair_for from library contract
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => Key::from(token_a),
            "token_b" => Key::from(token_b)
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );
        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let pair_package_hash: ContractPackageHash =
            runtime::call_versioned_contract(pair, None, "package_hash", runtime_args! {});

        // call transferFrom from IUniSwapV2Pair
        let args: RuntimeArgs = runtime_args! {
            "owner" => self.get_caller(),
            "recipient" => Key::from(pair_package_hash),
            "amount" => liquidity
        };

        let result: Result<(), u32> = Self::call_versioned_contract(
            &pair.to_formatted_string(),
            uniswapv2_contract_methods::PAIR_TRANSFER_FROM,
            args,
        );
        if result.is_err() {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed7 as u16,
            ));
        }

        // call burn from IUniSwapV2Pair
        let args: RuntimeArgs = runtime_args! {
            "to" => to,
        };
        let (amount0, amount1): (U256, U256) = Self::call_versioned_contract(
            &pair.to_formatted_string(),
            uniswapv2_contract_methods::PAIR_BURN,
            args,
        );

        // call sortTokens from library contract
        let args: RuntimeArgs = runtime_args! {
            "token_a" => Key::from(token_a),
            "token_b" => Key::from(token_b)
        };

        let (token0, _): (ContractPackageHash, ContractPackageHash) = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_SORT_TOKENS,
            args,
        );

        let (amount_a, amount_b): (U256, U256) = if token_a == token0 {
            (amount0, amount1)
        } else {
            (amount1, amount0)
        };

        if amount_a < amount_a_min || amount_b < amount_b_min {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort1 as u16));
        }
        self.emit(&ROUTEREvent::RemoveReserves {
            user: to,
            reserve0: amount_a,
            reserve1: amount_b,
            pair_contract_hash: pair,
        });
        (amount_a, amount_b)
    }

    fn remove_liquidity_cspr(
        &mut self,
        token: ContractPackageHash,
        liquidity: U256,
        amount_token_min: U256,
        amount_cspr_min: U256,
        to: Key,        // to's key to transfer back token
        to_purse: URef, // to's purse to transfer back cspr
    ) -> (U256, U256) {
        // calling self contract's removeLiquidity
        let package_hash = data::package_hash();
        let wcspr: ContractPackageHash = data::wcspr();

        let (amount_token, amount_cspr): (U256, U256) = self.remove_liquidity(
            token,
            wcspr,
            liquidity,
            amount_token_min,
            amount_cspr_min,
            Key::from(package_hash),
        );

        // transfer token to 'to'
        let result: Result<(), u32> =
            transfer_helper::safe_transfer(Key::from(token), to, amount_token);
        if result.is_err()
        // transfer failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed8 as u16,
            ));
        }

        // call withdraw and transfer cspr to 'to'
        let args: RuntimeArgs = runtime_args! {
            "to_purse" => to_purse,
            "amount" => U512::from(amount_cspr.as_u128())
        };

        let result: Result<(), u32> = Self::call_versioned_contract(
            &wcspr.to_formatted_string(),
            uniswapv2_contract_methods::WCSPR_WITHDRAW,
            args,
        );
        if result.is_err()
        // wcspr_withdraw failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed9 as u16,
            ));
        }
        (amount_token, amount_cspr)
    }

    fn remove_liquidity_with_permit(
        &mut self,
        token_a: ContractPackageHash,
        token_b: ContractPackageHash,
        liquidity: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        to: Key,
        approve_max: bool,
        public_key: String,
        signature: String,
        deadline: U256,
    ) -> (U256, U256) {
        let factory: ContractPackageHash = data::factory();

        // call pair_for method from uniswapv2Library
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => Key::from(token_a),
            "token_b" => Key::from(token_b)
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );
        let zero_addr: Key = Key::from_formatted_str(
            "hash-0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        if pair == zero_addr {
            // pair does not exist
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort2 as u16));
        }

        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let value: U256 = if approve_max { U256::MAX } else { liquidity };

        // call permit from uniswapv2pair
        let args: RuntimeArgs = runtime_args! {
            "public" => public_key,
            "signature" => signature,
            "owner" => self.get_caller(),
            "spender" => Key::from(data::package_hash()),
            "value" => value,
            "deadline" => deadline.as_u64()
        };

        let () = Self::call_versioned_contract(
            &pair.to_formatted_string(),
            uniswapv2_contract_methods::PAIR_PERMIT,
            args,
        );

        // call self remove_liquidity
        let (amount_a, amount_b): (U256, U256) =
            self.remove_liquidity(token_a, token_b, liquidity, amount_a_min, amount_b_min, to);
        (amount_a, amount_b)
    }

    fn remove_liquidity_cspr_with_permit(
        &mut self,
        token: ContractPackageHash,
        liquidity: U256,
        amount_token_min: U256,
        amount_cspr_min: U256,
        to: Key,
        approve_max: bool,
        public_key: String,
        signature: String,
        deadline: U256,
        to_purse: URef,
    ) -> (U256, U256) {
        let factory: ContractPackageHash = data::factory();
        let wcspr: ContractPackageHash = data::wcspr();

        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => Key::from(token),
            "token_b" => Key::from(wcspr)
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );
        let zero_addr: Key = Key::from_formatted_str(
            "hash-0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        if pair == zero_addr {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort3 as u16));
        }

        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let value: U256 = if approve_max { U256::MAX } else { liquidity };

        // call permit from uniswapv2pair
        let args: RuntimeArgs = runtime_args! {
            "public" => public_key,
            "signature" => signature,
            "owner" => self.get_caller(),
            "spender" => Key::from(data::package_hash()),
            "value" => value,
            "deadline" => deadline.as_u64()
        };
        let () = Self::call_versioned_contract(
            &pair.to_formatted_string(),
            uniswapv2_contract_methods::PAIR_PERMIT,
            args,
        );

        // call remove_liquidity_cspr

        let (amount_token, amount_cspr): (U256, U256) = self.remove_liquidity_cspr(
            token,
            liquidity,
            amount_token_min,
            amount_cspr_min,
            to,
            to_purse,
        );
        (amount_token, amount_cspr)
    }

    fn swap_exact_tokens_for_tokens(
        &mut self,
        amount_in: U256,
        amount_out_min: U256,
        _path: Vec<String>,
        to: Key,
    ) -> Vec<U256> {
        let factory: ContractPackageHash = data::factory();
        let mut path: Vec<Key> = Vec::new();
        for i in 0..(_path.len()) {
            path.push(Key::from_formatted_str(&_path[i]).unwrap());
        }
        // call getAmountsOut from Library contract
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "amount_in" => amount_in,
            "path" => path.clone(),
        };
        let amounts: Vec<U256> = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNTS_OUT,
            args,
        );

        if amounts[amounts.len() - 1] < amount_out_min {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort4 as u16));
        }

        // get pair
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => path[0],
            "token_b" => path[1],
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );
        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let pair_package_hash: ContractPackageHash =
            runtime::call_versioned_contract(pair, None, "package_hash", runtime_args! {});

        let result: Result<(), u32> = transfer_helper::safe_transfer_from(
            path[0],
            self.get_caller(),
            Key::from(pair_package_hash),
            amounts[0],
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed10 as u16,
            ));
        }

        Self::_swap(&amounts, &path, to);
        amounts
    }

    fn swap_tokens_for_exact_tokens(
        &mut self,
        amount_out: U256,
        amount_in_max: U256,
        _path: Vec<String>,
        to: Key,
    ) -> Vec<U256> {
        let factory: ContractPackageHash = data::factory();

        let mut path: Vec<Key> = Vec::new();
        for i in 0..(_path.len()) {
            path.push(Key::from_formatted_str(&_path[i]).unwrap());
        }
        // call getAmountIn from Library contract

        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "amount_out" => amount_out,
            "path" => path.clone(),
        };
        let amounts: Vec<U256> = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNTS_IN,
            args,
        );

        if amounts[0] > amount_in_max {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort5 as u16));
        }

        // Get pair
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => path[0],
            "token_b" => path[1],
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );

        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let pair_package_hash: ContractPackageHash =
            runtime::call_versioned_contract(pair, None, "package_hash", runtime_args! {});

        let result: Result<(), u32> = transfer_helper::safe_transfer_from(
            path[0],
            self.get_caller(),
            Key::from(pair_package_hash),
            amounts[0],
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed11 as u16,
            ));
        }

        Self::_swap(&amounts, &path, to);
        amounts
    }

    fn swap_exact_cspr_for_tokens(
        &mut self,
        amount_out_min: U256,
        amount_in: U256,
        _path: Vec<String>,
        to: Key,
        caller_purse: URef,
    ) -> Vec<U256> {
        let wcspr: ContractPackageHash = data::wcspr();
        let factory: ContractPackageHash = data::factory();
        let mut path: Vec<Key> = Vec::new();
        for i in 0..(_path.len()) {
            path.push(Key::from_formatted_str(&_path[i]).unwrap());
        }
        if !(path[0] == Key::from(wcspr)) {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort6 as u16));
        }

        // call get_amounts_out
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "amount_in" => amount_in,
            "path" => path.clone(),
        };
        let amounts: Vec<U256> = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNTS_OUT,
            args,
        );

        if amounts[amounts.len() - 1] < amount_out_min {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort7 as u16));
        }

        let self_purse = system::create_purse(); // create new temporary purse and transfer cspr from caller purse to this
        let _: () = system::transfer_from_purse_to_purse(
            caller_purse,
            self_purse,
            U512::from(amounts[0].as_u128()),
            None,
        )
        .unwrap_or_revert();

        let args: RuntimeArgs = runtime_args! {
            "amount" => U512::from(amounts[0].as_u128()),
            "purse" => self_purse,
        };
        let result: Result<(), u32> = Self::call_versioned_contract(
            &wcspr.to_formatted_string(),
            uniswapv2_contract_methods::WCSPR_DEPOSIT,
            args,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed13 as u16,
            ));
        }

        // call transfer method from IWETH
        // Get pair
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => path[0],
            "token_b" => path[1],
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );

        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let pair_package_hash: ContractPackageHash =
            runtime::call_versioned_contract(pair, None, "package_hash", runtime_args! {});

        let args: RuntimeArgs = runtime_args! {
            "recipient" => Key::from(pair_package_hash),
            "amount" => amounts[0]
        };

        let result: Result<(), u32> = Self::call_versioned_contract(
            &wcspr.to_formatted_string(),
            uniswapv2_contract_methods::WCSPR_TRANSFER,
            args,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed14 as u16,
            ));
        }

        Self::_swap(&amounts, &path, to);

        amounts
    }

    fn swap_tokens_for_exact_cspr(
        &mut self,
        amount_out: U256,
        amount_in_max: U256,
        _path: Vec<String>,
        to: URef, // recipient of cspr, must be a purse
    ) -> Vec<U256> {
        let wcspr: ContractPackageHash = data::wcspr();
        let factory: ContractPackageHash = data::factory();
        let self_addr: Key = Key::from(data::package_hash());
        let mut path: Vec<Key> = Vec::new();
        for i in 0..(_path.len()) {
            path.push(Key::from_formatted_str(&_path[i]).unwrap());
        }

        if !(path[path.len() - 1] == Key::from(wcspr)) {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort8 as u16));
        }

        // call getAmountIn from Library contract
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "amount_out" => amount_out,
            "path" => path.clone(),
        };
        let amounts: Vec<U256> = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNTS_IN,
            args,
        );

        if amounts[0] > amount_in_max {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort9 as u16));
        }

        // call safeTransferFrom from TransferHelper

        // first need to get the pair
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => path[0],
            "token_b" => path[1],
        };

        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );

        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let pair_package_hash: ContractPackageHash =
            runtime::call_versioned_contract(pair, None, "package_hash", runtime_args! {});

        let result: Result<(), u32> = transfer_helper::safe_transfer_from(
            path[0],
            self.get_caller(),
            Key::from(pair_package_hash),
            amounts[0],
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed15 as u16,
            ));
        }

        Self::_swap(&amounts, &path, self_addr);

        // call withdraw from WCSPR and transfer cspr to 'to'
        let args: RuntimeArgs = runtime_args! {
            "to_purse" => to,
            "amount" => U512::from(amounts[amounts.len() - 1].as_u128())
        };
        let result: Result<(), u32> = Self::call_versioned_contract(
            &wcspr.to_formatted_string(),
            uniswapv2_contract_methods::WCSPR_WITHDRAW,
            args,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed16 as u16,
            ));
        }

        amounts
    }

    fn swap_exact_tokens_for_cspr(
        &mut self,
        amount_in: U256,
        amount_out_min: U256,
        _path: Vec<String>,
        to: URef, // recipient of cspr, must be a purse
    ) -> Vec<U256> {
        let wcspr: ContractPackageHash = data::wcspr();
        let factory: ContractPackageHash = data::factory();
        let self_addr: Key = Key::from(data::package_hash());
        let mut path: Vec<Key> = Vec::new();
        for i in 0..(_path.len()) {
            path.push(Key::from_formatted_str(&_path[i]).unwrap());
        }

        if !(path[path.len() - 1] == Key::from(wcspr)) {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort10 as u16));
        }

        // call get_amounts_out
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "amount_in" => amount_in,
            "path" => path.clone(),
        };
        let amounts: Vec<U256> = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNTS_OUT,
            args,
        );

        if amounts[amounts.len() - 1] < amount_out_min {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort11 as u16));
        }

        // call safeTransferFrom from TransferHelper
        // first need to get the pair
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => path[0],
            "token_b" => path[1],
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );
        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let pair_package_hash: ContractPackageHash =
            runtime::call_versioned_contract(pair, None, "package_hash", runtime_args! {});

        let result: Result<(), u32> = transfer_helper::safe_transfer_from(
            path[0],
            self.get_caller(),
            Key::from(pair_package_hash),
            amounts[0],
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed17 as u16,
            ));
        }

        Self::_swap(&amounts, &path, self_addr);

        // call withdraw from WCSPR and transfer cspr to 'to'
        let args: RuntimeArgs = runtime_args! {
            "to_purse" => to,
            "amount" => U512::from(amounts[amounts.len() - 1].as_u128())
        };
        let result: Result<(), u32> = Self::call_versioned_contract(
            &wcspr.to_formatted_string(),
            uniswapv2_contract_methods::WCSPR_WITHDRAW,
            args,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed18 as u16,
            ));
        }

        amounts
    }

    fn swap_cspr_for_exact_tokens(
        &mut self,
        amount_out: U256,
        amount_in_max: U256,
        _path: Vec<String>,
        to: Key,
        caller_purse: URef,
    ) -> Vec<U256> {
        let wcspr: ContractPackageHash = data::wcspr();
        let factory: ContractPackageHash = data::factory();
        let mut path: Vec<Key> = Vec::new();
        for i in 0..(_path.len()) {
            path.push(Key::from_formatted_str(&_path[i]).unwrap());
        }
        if !(path[0] == Key::from(wcspr)) {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort12 as u16));
        }

        // call get_amounts_out
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "amount_out" => amount_out,
            "path" => path.clone(),
        };
        let amounts: Vec<U256> = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNTS_IN,
            args,
        );

        if amounts[0] > amount_in_max {
            runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterAbort13 as u16));
        }

        let self_purse = system::create_purse(); // create new temporary purse and transfer cspr from caller purse to this
        let _: () = system::transfer_from_purse_to_purse(
            caller_purse,
            self_purse,
            U512::from(amounts[0].as_u128()),
            None,
        )
        .unwrap_or_revert();

        // call deposit method from wcspr
        let args: RuntimeArgs = runtime_args! {
            "amount" => U512::from(amounts[0].as_u128()),
            "purse" => self_purse
        };
        let result: Result<(), u32> = Self::call_versioned_contract(
            &wcspr.to_formatted_string(),
            uniswapv2_contract_methods::WCSPR_DEPOSIT,
            args,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed20 as u16,
            ));
        }

        // call transfer method from wcspr
        // Get pair
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => path[0],
            "token_b" => path[1],
        };
        let pair: Key = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
            args,
        );

        let pair: ContractPackageHash =
            ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash
        let pair_package_hash: ContractPackageHash =
            runtime::call_versioned_contract(pair, None, "package_hash", runtime_args! {});

        let args: RuntimeArgs = runtime_args! {
            "recipient" => Key::from(pair_package_hash),
            "amount" => amounts[0]
        };
        let result: Result<(), u32> = Self::call_versioned_contract(
            &wcspr.to_formatted_string(),
            uniswapv2_contract_methods::WCSPR_TRANSFER,
            args,
        );
        if result.is_err()
        // transfer_from failed
        {
            runtime::revert(ApiError::User(
                ErrorCodes::UniswapV2RouterTransferFailed21 as u16,
            ));
        }

        Self::_swap(&amounts, &path, to);

        // No need to refund extra cspr because we are already getting the exact required amount from the purse
        amounts
    }

    fn quote(amount_a: U256, reserve_a: U256, reserve_b: U256) -> U256 {
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "amount_a" => amount_a,
            "reserve_a" => U128::from(reserve_a.as_u128()),
            "reserve_b" => U128::from(reserve_b.as_u128())
        };

        let amount_b: U256 = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_QUOTE,
            args,
        );
        amount_b
    }

    fn get_amount_out(amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "amount_in" => amount_in,
            "reserve_in" => reserve_in,
            "reserve_out" => reserve_out
        };

        let amount_out: U256 = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNT_OUT,
            args,
        );
        amount_out
    }

    fn get_amount_in(amount_out: U256, reserve_in: U256, reserve_out: U256) -> U256 {
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "amount_out" => amount_out,
            "reserve_in" => reserve_in,
            "reserve_out" => reserve_out
        };

        let amount_in: U256 = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNT_IN,
            args,
        );
        amount_in
    }

    fn get_amounts_out(amount_in: U256, path: Vec<Key>) -> Vec<U256> {
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let factory: ContractPackageHash = data::factory();

        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "amount_in" => amount_in,
            "path" => path
        };

        let amounts_out: Vec<U256> = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNTS_OUT,
            args,
        );
        amounts_out
    }

    fn get_amounts_in(amount_out: U256, path: Vec<Key>) -> Vec<U256> {
        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let factory: ContractPackageHash = data::factory();

        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "amount_out" => amount_out,
            "path" => path
        };

        let amounts_in: Vec<U256> = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_AMOUNTS_IN,
            args,
        );
        amounts_in
    }

    fn get_package_hash(&mut self) -> ContractPackageHash {
        data::package_hash()
    }

    // *************************************** Helper methods ****************************************

    fn _add_liquidity(
        token_a: ContractPackageHash,
        token_b: ContractPackageHash,
        amount_a_desired: U256,
        amount_b_desired: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        pair_received: Option<Key>,
    ) -> (U256, U256) {
        let factory: ContractPackageHash = data::factory();
        let args: RuntimeArgs = runtime_args! {
            "token0" => Key::from(token_a),
            "token1" => Key::from(token_b)
        };
        let pair: Key = Self::call_versioned_contract(
            &factory.to_formatted_string(),
            uniswapv2_contract_methods::FACTORY_GET_PAIR,
            args,
        );
        let zero_addr: Key = Key::from_formatted_str(
            "hash-0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut pair_already_exist: bool = false;

        // If a pair is not passed, check if it already exists, if it doesnot, revert
        if pair_received.is_none() {
            if pair == zero_addr {
                // if pair is none and it doesnot already exist, revert
                runtime::revert(ApiError::User(ErrorCodes::UniswapV2RouterZeroAddr as u16));
            } else {
                pair_already_exist = true;
            }
        }

        // If a pair is passed in, check if it exists already, if it does, no need to call factory's create_pair
        if pair_received.is_some() {
            if pair != zero_addr {
                pair_already_exist = true;
            }
        }

        if pair_already_exist == false {
            // need to call create_pair only once for each pair. If a same pair is passed again, no need to call this again
            let pair = pair_received.unwrap();
            let args = runtime_args! {
                "token_a" => Key::from(token_a),
                "token_b" => Key::from(token_b),
                "pair_hash" => Key::from(pair)
            };
            let _: () = Self::call_versioned_contract(
                &factory.to_formatted_string(),
                uniswapv2_contract_methods::FACTORY_CREATE_PAIR, // this create_pair method DOES NOT create a new pair, instead it initializes the pair passed in
                args,
            );
        }

        let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
        let args: RuntimeArgs = runtime_args! {
            "factory" => Key::from(factory),
            "token_a" => Key::from(token_a),
            "token_b" => Key::from(token_b),
        };
        let (reserve_a, reserve_b): (U128, U128) = Self::call_versioned_contract(
            &uniswapv2_library_contract_hash,
            uniswapv2_contract_methods::LIBRARY_GET_RESERVES,
            args,
        );

        if reserve_a == 0.into() && reserve_b == 0.into() {
            return (amount_a_desired, amount_b_desired);
        } else {
            let args: RuntimeArgs = runtime_args! {
                "amount_a" => amount_a_desired,
                "reserve_a" => reserve_a,
                "reserve_b" => reserve_b
            };

            let amount_b_optimal: U256 = Self::call_versioned_contract(
                &uniswapv2_library_contract_hash,
                uniswapv2_contract_methods::LIBRARY_QUOTE,
                args,
            );

            if amount_b_optimal <= amount_b_desired && amount_b_optimal >= amount_b_min {
                return (amount_a_desired, amount_b_optimal);
            } else {
                let args: RuntimeArgs = runtime_args! {
                    "amount_a" => amount_b_desired,
                    "reserve_a" => reserve_b,
                    "reserve_b" => reserve_a
                };
                let amount_a_optimal: U256 = Self::call_versioned_contract(
                    &uniswapv2_library_contract_hash,
                    uniswapv2_contract_methods::LIBRARY_QUOTE,
                    args,
                );

                if amount_a_optimal > amount_a_desired {
                    runtime::revert(ApiError::User(
                        ErrorCodes::UniswapV2RouterInvalidArguments as u16,
                    ));
                }

                if amount_a_optimal >= amount_a_min {
                    return (amount_a_optimal, amount_b_desired);
                } else {
                    return (0.into(), 0.into());
                }
            }
        }
    }

    fn _swap(amounts: &Vec<U256>, path: &Vec<Key>, _to: Key) {
        let factory = data::factory();
        for i in 0..(path.len() - 1)
        // start ≤ x < end - 1
        {
            let (input, output): (Key, Key) = (path[i], path[i + 1]);
            let args: RuntimeArgs = runtime_args! {
                "token_a" => input,
                "token_b" => output
            };

            let uniswapv2_library_contract_hash = data::library_hash().to_formatted_string();
            let (token0, _): (ContractPackageHash, ContractPackageHash) =
                Self::call_versioned_contract(
                    &uniswapv2_library_contract_hash,
                    uniswapv2_contract_methods::LIBRARY_SORT_TOKENS,
                    args,
                );

            let amount_out: U256 = amounts[i + 1];
            let (amount0_out, amount1_out): (U256, U256) = if input == Key::from(token0) {
                (0.into(), amount_out)
            } else {
                (amount_out, 0.into())
            };
            let to: Key = {
                if i < path.len() - 2 {
                    let args: RuntimeArgs = runtime_args! {
                        "factory" => Key::from(factory),
                        "token_a" => output,
                        "token_b" => path[i + 2]
                    };
                    let hash: Key = Self::call_versioned_contract(
                        &uniswapv2_library_contract_hash,
                        uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
                        args,
                    );
                    hash
                } else {
                    _to
                }
            };

            // Call swap from UniswapV2Pair, but first need to call pair_for to get the pair
            let args: RuntimeArgs = runtime_args! {
                "factory" => Key::from(factory),
                "token_a" => input,
                "token_b" => output
            };
            let pair: Key = Self::call_versioned_contract(
                &uniswapv2_library_contract_hash,
                uniswapv2_contract_methods::LIBRARY_PAIR_FOR,
                args,
            );
            let pair: ContractPackageHash =
                ContractPackageHash::from(pair.into_hash().unwrap_or_default()); // convert key into ContractPackageHash

            let args: RuntimeArgs = runtime_args! {
                "amount0_out" => amount0_out,
                "amount1_out" => amount1_out,
                "to" => to,
                "data" => "",
            };

            let () = Self::call_versioned_contract(
                &pair.to_formatted_string(),
                uniswapv2_contract_methods::PAIR_SWAP,
                args,
            );
        }
    }

    fn ensure(&self, deadline: U256) -> bool {
        // shadowing the variable
        let deadline = BlockTime::new(deadline.as_u64());
        let blocktime = runtime::get_blocktime();

        deadline >= blocktime
    }

    fn call_versioned_contract<T: CLTyped + FromBytes>(
        package_hash_str: &str,
        method: &str,
        args: RuntimeArgs,
    ) -> T {
        let package_hash = ContractPackageHash::from_formatted_str(package_hash_str);
        runtime::call_versioned_contract(package_hash.unwrap_or_default(), None, method, args)
    }
    fn emit(&mut self, router_event: &ROUTEREvent) {
        let mut events = Vec::new();
        let package = self.get_package_hash();
        match router_event {
            ROUTEREvent::AddReserves {
                user,
                reserve0,
                reserve1,
                pair_contract_hash,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", router_event.type_name());
                event.insert("user", user.to_string());
                event.insert("reserve0", reserve0.to_string());
                event.insert("reserve1", reserve1.to_string());
                event.insert("pair_contract_hash", pair_contract_hash.to_string());
                events.push(event);
            }
            ROUTEREvent::RemoveReserves {
                user,
                reserve0,
                reserve1,
                pair_contract_hash,
            } => {
                let mut event = BTreeMap::new();
                event.insert("contract_package_hash", package.to_string());
                event.insert("event_type", router_event.type_name());
                event.insert("user", user.to_string());
                event.insert("reserve0", reserve0.to_string());
                event.insert("reserve1", reserve1.to_string());
                event.insert("pair_contract_hash", pair_contract_hash.to_string());
                events.push(event);
            }
        };
        for event in events {
            let _: URef = storage::new_uref(event);
        }
    }
}
