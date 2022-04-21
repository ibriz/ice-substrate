//! Benchmarking setup for pallet-template

use super::*;

use crate as pallet_airdrop;
#[allow(unused)]
use crate::Pallet;
use codec::Decode;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_core::*;
use sp_runtime::traits::Saturating;
use sp_std::prelude::*;
use types::AccountIdOf;
use types::BlockNumberOf;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {


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

    dispatch_user_claim {
        let caller: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();
        <OffchainAccount<T>>::set(Some(caller.clone()));
        let ice_bytes = hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

        let ice_address = T::AccountId::decode(&mut &ice_bytes[..]).unwrap_or_default();

        let icon_address:[u8; 20] = hex_literal::hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9");

        let icon_signature:[u8;65] = hex_literal::hex!("628af708622383d60e1d9d95763cf4be64d0bafa8daebb87847f14fde0db40013105586f0c937ddf0e8913251bf01cf8e0ed82e4f631b666453e15e50d69f3b900");

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
        pallet_airdrop::ExchangeAccountsMap::<T>::insert(&icon_address,true);

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

	update_airdrop_state {
		let old_state= Pallet::<T>::get_airdrop_state();
		let new_state = types::AirdropState::default();

	}: update_airdrop_state(RawOrigin::Root, new_state.clone())
	verify {
         assert_last_event::<T>(Event::AirdropStateUpdated {
			old_state,
			new_state,
		}.into());
	}


	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
