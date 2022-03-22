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
use sp_runtime::traits::Saturating;
use sp_core::*;
use log;
use serde::Deserialize;
use codec::alloc::string::String;
use codec::{Decode, Encode};
use sp_runtime::traits::TrailingZeroInput;


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

    complete_transfer_success {
       
        let x in 0 .. 10000;
        let b in 0 .. 10000 ;
        let c in 0 .. 10000 ;
        let bl_number= BlockNumberOf::<T>::from(b);
        let system_account_id = Pallet::<T>::get_creditor_account();

        let icon_address:types::IconAddress = account("icon_address", 0, x);
       
        
       
        let claimer : AccountIdOf<T> = AccountIdOf::<T>::default();
        let response =types::ServerResponse {
            omm: 123_u32.into(),
            amount: 10_u32.into(),
            stake: 12_u32.into(),
            defi_user: true,
        };

        Pallet::<T>::init_balance(&system_account_id,10_00_00_00);
        Pallet::<T>::setup_claimer(claimer.clone(),bl_number,icon_address.clone());
       
       
    }:complete_transfer(RawOrigin::Root,bl_number,claimer.clone(),response)
	 verify {
	    assert_last_event::<T>(Event::ClaimSuccess(claimer).into());
    }


    donate_to_creditor{
        let x in 10 .. 100;

        let caller: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();
        let system_account_id = Pallet::<T>::get_creditor_account();
        Pallet::<T>::init_balance(&system_account_id,10_00_00_00);
        Pallet::<T>::init_balance(&caller,10_00_00_00_00);
        let amount= types::BalanceOf::<T>::from(x);

    }:donate_to_creditor(RawOrigin::Signed(caller.clone()),amount.clone(),false)
    verify {
        assert_last_event::<T>(Event::DonatedToCreditor(caller,amount.clone()).into());
    }

    register_failed_claim {
        let x in 0 .. 10000;
        let b in 10 .. 10000 ;
        let ice_address : AccountIdOf<T> = account("ice_address", 0, x);
        let block_number = BlockNumberOf::<T>::from(b);
        let new_block_number=block_number.saturating_add(3u32.into());
        pallet_airdrop::PendingClaims::<T>::insert(block_number, &ice_address, 3u8);
        

    }:register_failed_claim(RawOrigin::Root,block_number,ice_address.clone())
    verify {
        assert_last_event::<T>(Event::RegisteredFailedClaim(ice_address.clone(),new_block_number.clone(),2u8).into());
    }

    claim_request {
        let x in 0 .. 10000;
       // let ice_address : AccountIdOf<T> = account("ice_address", 0, x);
        let ice_bytes = hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");
        let ice_address :AccountIdOf<T> =Decode::decode(&mut TrailingZeroInput::new(ice_bytes.as_ref())).unwrap();
		let icon_address = hex_literal::hex!("000000000000000000000000000000").to_vec();
        let icon_signature = hex_literal::hex!("628af708622383d60e1d9d95763cf4be64d0bafa8daebb87847f14fde0db40013105586f0c937ddf0e8913251bf01cf8e0ed82e4f631b666453e15e50d69f3b900").to_vec();
        let message = (*"icx_sendTransaction.data.{method.transfer.params.{wallet.da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c}}.dataType.call.from.hxdd9ecb7d3e441d25e8c4f03cd20a80c502f0c374.nid.0x1.nonce.0x1..timestamp.0x5d56f3231f818.to.cx8f87a4ce573a2e1377545feabac48a960e8092bb.version.0x3").as_bytes().to_vec();
        let snapshot = types::SnapshotInfo::<T>::default();

    }:claim_request(RawOrigin::Signed(ice_address.clone()),icon_address.clone(),message,icon_signature.clone())
    verify {
        assert_last_event::<T>(Event::ClaimRequestSucceeded(ice_address.clone()).into());
    }




	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
