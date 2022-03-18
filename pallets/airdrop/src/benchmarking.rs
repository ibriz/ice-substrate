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
use pallet_balances::Pallet as Balances;


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

    }:remove_from_pending_queue(RawOrigin::Root,bl_number,ice_address.clone())
	 verify {
	  assert_last_event::<T>(Event::RemovedFromQueue(ice_address).into());
    }

    // complete_transfer_success {
       
    //     let x in 0 .. 10000;
    //     let b in 0 .. 10000 ;
    //     let c in 0 .. 10000 ;
    //     let bl_number= BlockNumberOf::<T>::from(b);
    //     let system_account_id = Pallet::<T>::get_creditor_account();
    //     let icon_address = account("icon_address", 0, x);
       
        
       
    //     let claimer : AccountIdOf<T> = account("claimer", 0, x);
    //     let mut response =types::ServerResponse::default();
    //     response.amount=1_u128;
    //     // pallet_airdrop::PendingClaims::<T>::insert(bl_number, &claimer, 1_u8);

    //     Pallet::<T>::init_balance(10_00_00);
    //     Pallet::<T>::setup_claimer(&claimer,bl_number);

	// 	let snapshot = types::SnapshotInfo::<T>::default();
        
    //     pallet_airdrop::IceSnapshotMap::<T>::insert(claimer.clone(), snapshot.clone().icon_address(icon_address));
       
    // }:complete_transfer(RawOrigin::Root,bl_number,claimer.clone(),response)
	//  verify {
	//     assert_last_event::<T>(Event::RetryExceed(claimer, bl_number).into());
    // }


    // donate_to_creditor{
    //     let x in 0 .. 100;

    //     let caller: <T as frame_system::Config>::AccountId = frame_benchmarking::whitelisted_caller();
    //     let amount= types::BalanceOf<T>::from(x);

        

    // }:donate_to_creditor(RawOrigin::Signed(caller),amount.clone(),true)
    // verify {
    //     assert_last_event::<T>(Event::DonatedToCreditor(caller,amount.clone()).into());
    // }




	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
