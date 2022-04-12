//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet;
use crate as pallet_airdrop;
use frame_benchmarking::{benchmarks,account,whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::prelude::*;
use types::AccountIdOf;
use types::BlockNumberOf;
use sp_runtime::traits::Saturating;
use sp_core::*;
use codec::{Decode};



fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}



benchmarks! {
	
    remove_from_pending_queue{
       
        let b in 0 .. 10000 ;
        let u in 10 .. 25;

        let bl_number= BlockNumberOf::<T>::from(b);

        let icon_address : [u8;20] = [u as u8;20];

        pallet_airdrop::PendingClaims::<T>::insert(bl_number, icon_address.clone(), 1_u8);
       
        

    }:remove_from_pending_queue(RawOrigin::Root,bl_number,icon_address.clone())

	 verify {

	  assert_last_event::<T>(Event::RemovedFromQueue(icon_address).into());

    }

    complete_transfer{
       
        let b in 0 .. 10000 ;
        let u in 25 .. 100;

        let bl_number= BlockNumberOf::<T>::from(b);

        let system_account_id = Pallet::<T>::get_creditor_account();

        let icon_address : [u8;20] = [u as u8;20];
       
        let claimer : AccountIdOf<T>= frame_benchmarking::whitelisted_caller();

        let response =types::ServerResponse {
            omm: 123_u32.into(),
            amount: 10_u32.into(),
            stake: 12_u32.into(),
            defi_user: true,
        };

        Pallet::<T>::init_balance(&system_account_id,10_00_00_00);
        Pallet::<T>::setup_claimer(claimer.clone(),bl_number,icon_address.clone());
       
       
    }:complete_transfer(RawOrigin::Root,bl_number,icon_address.clone(),response)

	 verify {

	    assert_last_event::<T>(Event::ClaimSuccess(icon_address.clone()).into());
        let snapshot = Pallet::<T>::get_icon_snapshot_map(&icon_address).unwrap();
        assert_eq!(snapshot.claim_status,true);
    }


    donate_to_creditor{
        let x in 10 .. 100;

        let caller: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();

        let system_account_id = Pallet::<T>::get_creditor_account();

        Pallet::<T>::init_balance(&system_account_id,10_00_00_00);

        Pallet::<T>::init_balance(&caller,10_00_00_00_00);
        
        let amount = types::BalanceOf::<T>::from(x);

    }:donate_to_creditor(RawOrigin::Signed(caller.clone()),amount.clone(),false)

    verify {

        assert_last_event::<T>(Event::DonatedToCreditor(caller,amount).into());
    }

    register_failed_claim {

        let b in 10 .. 10000 ;
        let u in 50 .. 100;

        let icon_address : [u8;20] = [u as u8;20];

        let block_number = BlockNumberOf::<T>::from(b);

        let claimer: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();

        Pallet::<T>::setup_claimer(claimer.clone(),block_number,icon_address.clone());
        

    }:register_failed_claim(RawOrigin::Root,block_number,icon_address.clone())

    verify {
        let new_block_number = BlockNumberOf::<T>::from(2_u32);
        assert_last_event::<T>(Event::RegisteredFailedClaim(claimer.clone(),new_block_number.clone(),2u8).into());
    }

    claim_request {

        let ice_bytes = hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

        let ice_address = T::AccountId::decode(&mut &ice_bytes[..]).unwrap_or_default();

        let icon_address:[u8; 20] = hex_literal::hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9");

        let icon_signature = hex_literal::hex!("628af708622383d60e1d9d95763cf4be64d0bafa8daebb87847f14fde0db40013105586f0c937ddf0e8913251bf01cf8e0ed82e4f631b666453e15e50d69f3b900").to_vec();

        let message = (*"icx_sendTransaction.data.{method.transfer.params.{wallet.da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c}}.dataType.call.from.hxdd9ecb7d3e441d25e8c4f03cd20a80c502f0c374.nid.0x1.nonce.0x1..timestamp.0x5d56f3231f818.to.cx8f87a4ce573a2e1377545feabac48a960e8092bb.version.0x3").as_bytes().to_vec();


    }:claim_request(RawOrigin::Signed(ice_address.clone()),icon_address.clone(),message,icon_signature.clone())

    verify {

        assert_last_event::<T>(Event::ClaimRequestSucceeded{
            ice_address:ice_address.clone(),
            icon_address: icon_address.clone(),
            registered_in: Pallet::<T>::get_current_block_number()
        }.into());
    }

    update_processed_upto_counter {
        let b in 10 .. 10000;
        let block_number = BlockNumberOf::<T>::from(b);

    }:update_processed_upto_counter(RawOrigin::Root,block_number.clone())
    verify {
        assert_last_event::<T>(Event::ProcessedCounterSet(block_number.clone()).into());
    }

    set_offchain_account {
        let old_account: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();

        let new_account: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();

        <OffchainAccount<T>>::set(Some(old_account.clone()));

    }: set_offchain_account(RawOrigin::Root,new_account.clone())
    verify {
        assert_last_event::<T>(Event::OffchainAccountChanged{
            old_account:Some(old_account.clone()),
            new_account:new_account.clone()
        }.into());
    }

    force_claim_request {
        let u in 50 .. 100;

        let claimer: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();

        let icon_address : [u8;20] = [u as u8;20];

    }:force_claim_request(RawOrigin::Root,claimer.clone(),icon_address.clone())
    verify {

        assert_last_event::<T>(Event::ClaimRequestSucceeded{
            ice_address:claimer.clone(),
            icon_address: icon_address.clone(),
            registered_in: Pallet::<T>::get_current_block_number()
        }.into());

    }

    dispatch_user_claim {
        let caller: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();
        <OffchainAccount<T>>::set(Some(caller.clone()));
        let ice_bytes = hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

        let ice_address = T::AccountId::decode(&mut &ice_bytes[..]).unwrap_or_default();

        let icon_address:[u8; 20] = hex_literal::hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9");

        let icon_signature = hex_literal::hex!("628af708622383d60e1d9d95763cf4be64d0bafa8daebb87847f14fde0db40013105586f0c937ddf0e8913251bf01cf8e0ed82e4f631b666453e15e50d69f3b900").to_vec();

        let message = (*"icx_sendTransaction.data.{method.transfer.params.{wallet.da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c}}.dataType.call.from.hxdd9ecb7d3e441d25e8c4f03cd20a80c502f0c374.nid.0x1.nonce.0x1..timestamp.0x5d56f3231f818.to.cx8f87a4ce573a2e1377545feabac48a960e8092bb.version.0x3").as_bytes().to_vec();
        let server_data =types::ServerResponse {
            omm: 123_u32.into(),
            amount: 10_u32.into(),
            stake: 12_u32.into(),
            defi_user: true,
        };
        let system_account_id = Pallet::<T>::get_creditor_account();

        Pallet::<T>::init_balance(&system_account_id,10_00_00_00);
        Pallet::<T>::init_balance(&ice_address,10_00_00_00);



    }: dispatch_user_claim(RawOrigin::Signed(caller.clone()),icon_address.clone(),ice_address,message.to_vec(),icon_signature,server_data)
    verify {
        assert_last_event::<T>(Event::ClaimSuccess(icon_address.clone()).into());
    }


    dispatch_exchange_claim {
      
        let ice_bytes = hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

        let ice_address = T::AccountId::decode(&mut &ice_bytes[..]).unwrap_or_default();

        let icon_address:[u8; 20] = hex_literal::hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9");

        let server_data =types::ServerResponse {
            omm: 123_u32.into(),
            amount: 10_u32.into(),
            stake: 12_u32.into(),
            defi_user: true,
        };
        let system_account_id = Pallet::<T>::get_creditor_account();

        Pallet::<T>::init_balance(&system_account_id,10_00_00_00);
        Pallet::<T>::init_balance(&ice_address,10_00_00_00);



    }: dispatch_exchange_claim(RawOrigin::Root,icon_address.clone(),ice_address,server_data)
    verify {
        assert_last_event::<T>(Event::ClaimSuccess(icon_address.clone()).into());
    }


    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
