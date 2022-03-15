//! Benchmarking setup for pallet-template

use super::*;

// use super::Event;

#[allow(unused)]
use crate::Pallet;
use crate as pallet_airdrop;
use frame_benchmarking::{benchmarks,account};
use frame_system::RawOrigin;
// use frame_system::Config;
use sp_std::prelude::*;
use types::AccountIdOf;
use types::BlockNumberOf;
use core::str::FromStr;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	
    remove_from_pending_queue{
       
        let x in 0 .. 10000;
        let b in 0 .. 10000 ;
        let bl_number= BlockNumberOf::<T>::from(b);
        let caller: <T as frame_system::Config>::AccountId = frame_benchmarking::whitelisted_caller();
        let ice_address : AccountIdOf<T> = account("ice_address", 0, x);

        pallet_airdrop::PendingClaims::<T>::insert(bl_number, &ice_address, 1_u8);

    }:remove_from_pending_queue(RawOrigin::Signed(caller.clone()),bl_number,ice_address.clone())
	 verify {
	  assert_last_event::<T>(Event::RemovedFromQueue(ice_address).into());
    }

    complete_transfer_success{
       
        let x in 0 .. 10000;
        let b in 0 .. 10000 ;
        let bl_number= BlockNumberOf::<T>::from(b);
        let caller: <T as frame_system::Config>::AccountId = frame_benchmarking::whitelisted_caller();
        let receiver : AccountIdOf<T> = account("receiver", 0, x);
        let mut response =types::ServerResponse::default();
        response.amount=1_u128;

    }:complete_transfer(RawOrigin::Signed(caller.clone()),bl_number,receiver.clone(),response)
	 verify {
	  assert_last_event::<T>(Event::ClaimSuccess(receiver).into());
    }




	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
