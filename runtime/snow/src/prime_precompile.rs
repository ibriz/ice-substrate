#![cfg_attr(not(feature = "std"), no_std)]

use pallet_evm::{AddressMapping,Precompile,PrecompileOutput, PrecompileSet, PrecompileFailure, PrecompileResult, PrecompileHandle, ExitSucceed, ExitError};

extern crate alloc;

use alloc::{vec, vec::Vec};

pub struct PrimeTest;

fn estimate_gas_cost(num : &u128) -> u64 {
    (*num as f64) as u64
}

fn test_prime(num : &u128) -> bool {
    for i in 2..num/2 {
        if num % i == 0 {
            return false;
        }
    }
    true
}


impl Precompile for PrimeTest {

    fn execute(handle : &mut impl PrecompileHandle) -> PrecompileResult {
        let input = handle.input();

        let val = u128::from_be_bytes(input[0..16].try_into().expect("Expected input of exactly 16 length"));
        let gas_cost = estimate_gas_cost(&val);

        handle.record_cost(gas_cost)?;

        let vec;
        if test_prime(&val) {
            vec = vec![1];
        }
        else {
            vec = vec![0];
        }

        Ok(PrecompileOutput {
            exit_status : ExitSucceed::Returned,
            output : vec.to_vec()
        })
    }
}
