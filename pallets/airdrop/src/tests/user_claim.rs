use super::prelude::*;
use crate::tests::UserClaimTestCase;
use frame_support::traits::{Currency, LockableCurrency, ReservableCurrency};

#[test]
fn claim_success() {
	let ofw_account = samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));
		let mut case = UserClaimTestCase::default();
		case.amount = 12_017_332_u64.into();

		set_creditor_balance(10_000_0000);

		assert_ok!(AirdropModule::dispatch_user_claim(
			Origin::signed(AirdropModule::get_airdrop_server_account().unwrap()),
			case.icon_address,
			case.ice_address,
			case.message,
			case.icon_signature,
			case.ice_signature,
			case.amount,
			case.defi_user,
			case.merkle_proofs,
		));
		let ice_account = AirdropModule::to_account_id(case.ice_address.clone()).unwrap();
		let claim_balance = <Test as pallet_airdrop::Config>::Currency::usable_balance(ice_account);

		#[cfg(not(feature = "no-vesting"))]
		assert_eq!(claim_balance, 6761333);

		#[cfg(feature = "no-vesting")]
		assert_eq!(claim_balance, case.amount);

		let snapshot = <pallet_airdrop::IconSnapshotMap<Test>>::get(&case.icon_address).unwrap();

		let mapped_icon_wallet = AirdropModule::get_ice_to_icon_map(&ice_account);
		assert_eq!(mapped_icon_wallet, Some(case.icon_address));

		assert_eq!(snapshot.done_instant, true);
		#[cfg(not(feature = "no-vesting"))]
		assert_eq!(snapshot.done_vesting, true);
	});
}

#[test]
fn insufficient_balance() {
	let ofw_account = samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));

		let mut case = UserClaimTestCase::default();
		case.amount = 10017332_u64.into();
		let creditor_account = AirdropModule::get_creditor_account();
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_u32.into(),
			10_u32.into(),
		)
		.unwrap();

		assert_err!(
			AirdropModule::dispatch_user_claim(
				Origin::signed(AirdropModule::get_airdrop_server_account().unwrap()),
				case.icon_address,
				case.ice_address.clone(),
				case.message,
				case.icon_signature,
				case.ice_signature,
				case.amount,
				case.defi_user,
				case.merkle_proofs
			),
			PalletError::InsufficientCreditorBalance
		);
	});
}

#[test]
fn already_claimed() {
	let ofw_account = samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));
		let mut case = UserClaimTestCase::default();
		case.amount = 10017332_u64.into();

		let mut snapshot = types::SnapshotInfo::default().ice_address(case.ice_address.clone());
		snapshot.done_instant = true;
		snapshot.done_vesting = true;

		pallet_airdrop::IconSnapshotMap::<Test>::insert(&case.icon_address, snapshot);
		let creditor_account = AirdropModule::get_creditor_account();

		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

		assert_err!(
			AirdropModule::dispatch_user_claim(
				Origin::signed(AirdropModule::get_airdrop_server_account().unwrap()),
				case.icon_address,
				case.ice_address.clone(),
				case.message,
				case.icon_signature,
				case.ice_signature,
				case.amount,
				case.defi_user,
				case.merkle_proofs
			),
			PalletError::ClaimAlreadyMade
		);
	});
}

#[test]
fn invalid_payload() {
	let ofw_account = samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
        assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));
		let mut case =UserClaimTestCase::default();

		case.message = *b"icx_sendTransaction.data.{method.transfer.params.{wallet.eee7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48}}.dataType.call.from.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.version.0x3";
		let creditor_account = AirdropModule::get_creditor_account();

		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

		assert_err!(
			AirdropModule::dispatch_user_claim(
				Origin::signed(AirdropModule::get_airdrop_server_account().unwrap()),
				case.icon_address,
				case.ice_address.clone(),
				case.message,
				case.icon_signature,
				case.ice_signature,
				case.amount,
				case.defi_user,
				case.merkle_proofs
			),
			PalletError::InvalidMessagePayload
		);
	});
}

#[test]
fn invalid_ice_signature() {
	let ofw_account = samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));
		let mut case = UserClaimTestCase::default();
		case.ice_signature = [0u8; 64];

		let creditor_account = AirdropModule::get_creditor_account();
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

		assert_err!(
			AirdropModule::dispatch_user_claim(
				Origin::signed(AirdropModule::get_airdrop_server_account().unwrap()),
				case.icon_address,
				case.ice_address.clone(),
				case.message,
				case.icon_signature,
				case.ice_signature,
				case.amount,
				case.defi_user,
				case.merkle_proofs
			),
			PalletError::InvalidIceSignature
		);
	});
}

#[test]
fn invalid_icon_signature() {
	let ofw_account = samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));
		let mut case = UserClaimTestCase::default();
		case.icon_signature = [0u8; 65];

		let creditor_account = AirdropModule::get_creditor_account();
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

		assert_err!(
			AirdropModule::dispatch_user_claim(
				Origin::signed(AirdropModule::get_airdrop_server_account().unwrap()),
				case.icon_address,
				case.ice_address.clone(),
				case.message,
				case.icon_signature,
				case.ice_signature,
				case.amount,
				case.defi_user,
				case.merkle_proofs
			),
			PalletError::InvalidSignature
		);
	});
}

#[test]
fn partail_transfer_can_reclaim() {
	minimal_test_ext().execute_with(|| {
		// We are at block 1 now
		run_to_block(1);

		let mut case = UserClaimTestCase::default();
		case.amount = 10_u64.pow(18).into();
		let ice_account = AirdropModule::to_account_id(case.ice_address.clone()).unwrap();
		set_creditor_balance(Bounded::max_value());

		let mut user_balance =
			<Test as pallet_airdrop::Config>::Currency::total_balance(&ice_account);
		assert_eq!(user_balance, 0u32.into());

		#[cfg(feature = "no-vesting")]
		let (instant_amount, vesting_amount) = (case.amount, 0u128);

		#[cfg(not(feature = "no-vesting"))]
		let (instant_amount, vesting_amount) = {
			let (raw_instant, raw_vesting) =
				utils::get_splitted_amounts::<Test>(case.amount, case.defi_user).unwrap();
			let (schedule, rem) = utils::new_vesting_with_deadline::<
				Test,
				{ crate::vested_transfer::VESTING_APPLICABLE_FROM },
			>(raw_vesting, crate::vested_transfer::BLOCKS_IN_YEAR.into());

			(raw_instant + rem, schedule.unwrap().locked())
		};

		// Eat all vesting slots so next vesting will fail
		{
			let vesting_count_limit = <Test as pallet_vesting::Config>::MAX_VESTING_SCHEDULES;
			let mut amount_consumed = 0;
			for i in 0..vesting_count_limit {
				let res = pallet_vesting::Pallet::<Test>::vested_transfer(
					Origin::signed(AirdropModule::get_creditor_account()),
					ice_account.clone(),
					types::VestingInfoOf::<Test>::new(10_000, 2000, 5),
				)
				.map(|_| i);
				assert_eq!(res, Ok(i));
				amount_consumed += 10_000;
			}

			let new_balance =
				<Test as pallet_airdrop::Config>::Currency::total_balance(&ice_account);
			assert_eq!(new_balance, user_balance + amount_consumed);
			user_balance = new_balance;
		}

		// Try to claim in first attempt.
		// It will only let instant transfer to pass
		{
			let case = case.clone();
			assert_ok!(AirdropModule::dispatch_user_claim(
				Origin::root(),
				case.icon_address,
				case.ice_address,
				case.message,
				case.icon_signature,
				case.ice_signature,
				case.amount,
				case.defi_user,
				case.merkle_proofs,
			));

			let snapshot = AirdropModule::get_icon_snapshot_map(&case.icon_address).unwrap();
			let mapped_icon_wallet = AirdropModule::get_ice_to_icon_map(&ice_account);
			let new_balance =
				<Test as pallet_airdrop::Config>::Currency::total_balance(&ice_account);
			assert_eq!(true, snapshot.done_instant);
			assert_eq!(false, snapshot.done_vesting);
			assert_eq!(mapped_icon_wallet.as_ref(), Some(&case.icon_address));
			assert_eq!(new_balance, user_balance + instant_amount);

			user_balance = new_balance;
		}

		// Release the streames vesting schedules put previously
		{
			run_to_block(12);
			assert_ok!(pallet_vesting::Pallet::<Test>::vest(Origin::signed(
				ice_account.clone()
			)));
		}

		// Try to claim again
		#[cfg(not(feature = "no-vesting"))]
		{
			assert_ok!(AirdropModule::dispatch_user_claim(
				Origin::root(),
				case.icon_address,
				case.ice_address,
				case.message,
				case.icon_signature,
				case.ice_signature,
				case.amount,
				case.defi_user,
				case.merkle_proofs,
			));

			let snapshot = AirdropModule::get_icon_snapshot_map(&case.icon_address).unwrap();
			let mapped_icon_wallet = AirdropModule::get_ice_to_icon_map(&ice_account);
			let new_balance =
				<Test as pallet_airdrop::Config>::Currency::total_balance(&ice_account);
			assert_eq!(true, snapshot.done_instant);
			assert_eq!(true, snapshot.done_vesting);
			assert_eq!(mapped_icon_wallet.as_ref(), Some(&case.icon_address));
			assert_eq!(new_balance, user_balance + vesting_amount);
		}

		#[cfg(feature = "no-vesting")]
		{
			assert_noop!(
				AirdropModule::dispatch_user_claim(
					Origin::root(),
					case.icon_address,
					case.ice_address,
					case.message,
					case.icon_signature,
					case.ice_signature,
					case.amount,
					case.defi_user,
					case.merkle_proofs,
				),
				PalletError::ClaimAlreadyMade
			);
		}
	});
}
