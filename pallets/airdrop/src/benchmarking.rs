//! Benchmarking setup for pallet-template

// use super::*;

#[allow(unused)]
use crate::Pallet;
use crate as pallet_airdrop;
use frame_benchmarking::{benchmarks};
use frame_system::RawOrigin;
use frame_system::Config;

benchmarks! {
	
    benchmark_test {
        
        let caller: <T as frame_system::Config>::AccountId = frame_benchmarking::whitelisted_caller();
    }:{

	} verify {
		
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
