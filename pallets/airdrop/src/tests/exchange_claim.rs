use frame_support::{traits::ConstU32, BoundedVec};

use crate::tests::to_test_case;

use super::prelude::*;
const VALID_ICON_WALLET: types::IconAddress =
	decode_hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9");

#[test]
fn claim_success() {
	let sample = samples::MERKLE_PROOF_SAMPLE;
	let case = to_test_case(sample);
	let bounded_proofs = BoundedVec::<types::MerkleHash, ConstU32<10>>::try_from(case.1).unwrap();
	let defi_user = true;
	let amount: types::BalanceOf<Test> = 10017332_u64.into();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		let icon_wallet = VALID_ICON_WALLET;
		let ice_address =
			hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

		let creditor_account = AirdropModule::get_creditor_account();
		pallet_airdrop::ExchangeAccountsMap::<Test>::insert(icon_wallet, amount);
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

		assert_ok!(AirdropModule::dispatch_exchange_claim(
			Origin::root(),
			icon_wallet,
			ice_address.clone(),
			amount.into(),
			defi_user,
			bounded_proofs,
		));
	});
}

#[test]
fn insufficient_balance() {
	let sample = samples::MERKLE_PROOF_SAMPLE;
	let case = to_test_case(sample);
	let bounded_proofs = BoundedVec::<types::MerkleHash, ConstU32<10>>::try_from(case.1).unwrap();
	let defi_user = true;
	let amount: types::BalanceOf<Test> = 10017332_u64.into();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		let icon_wallet = VALID_ICON_WALLET;
		let ice_address =
			hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

		let creditor_account = AirdropModule::get_creditor_account();
		pallet_airdrop::ExchangeAccountsMap::<Test>::insert(&icon_wallet, amount);
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_u32.into(),
			10_u32.into(),
		)
		.unwrap();

		assert_err!(
			AirdropModule::dispatch_exchange_claim(
				Origin::root(),
				icon_wallet,
				ice_address.clone(),
				amount,
				defi_user,
				bounded_proofs,
			),
			PalletError::InsufficientCreditorBalance
		);
	});
}
#[test]
fn already_claimed() {
	let sample = samples::MERKLE_PROOF_SAMPLE;
	let case = to_test_case(sample);
	let bounded_proofs = BoundedVec::<types::MerkleHash, ConstU32<10>>::try_from(case.1).unwrap();
	let defi_user = true;
	let amount: types::BalanceOf<Test> = 10017332_u64.into();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		let icon_wallet = VALID_ICON_WALLET;
		let ice_address =
			hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

		let mut snapshot = types::SnapshotInfo::default();
		snapshot.done_instant = true;
		snapshot.done_vesting = true;

		pallet_airdrop::IceSnapshotMap::<Test>::insert(&icon_wallet, snapshot);
		let creditor_account = AirdropModule::get_creditor_account();
		pallet_airdrop::ExchangeAccountsMap::<Test>::insert(&icon_wallet, amount);
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

		assert_err!(
			AirdropModule::dispatch_exchange_claim(
				Origin::root(),
				icon_wallet,
				ice_address.clone(),
				amount,
				defi_user,
				bounded_proofs,
			),
			PalletError::ClaimAlreadyMade
		);
	});
}

#[test]
fn only_whitelisted_claim() {
	let sample = samples::MERKLE_PROOF_SAMPLE;
	let case = to_test_case(sample);
	let bounded_proofs = BoundedVec::<types::MerkleHash, ConstU32<10>>::try_from(case.1).unwrap();
	let defi_user = true;
	let amount: types::BalanceOf<Test> = 10017332_u64.into();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		let icon_wallet = VALID_ICON_WALLET;
		let ice_address =
			hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

		let snapshot = types::SnapshotInfo::default();

		pallet_airdrop::IceSnapshotMap::<Test>::insert(&icon_wallet, snapshot);
		let creditor_account = AirdropModule::get_creditor_account();
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

		assert_err!(
			AirdropModule::dispatch_exchange_claim(
				Origin::root(),
				icon_wallet,
				ice_address.clone(),
				amount,
				defi_user,
				bounded_proofs,
			),
			PalletError::DeniedOperation
		);
	});
}

#[test]
fn invalid_claim_amount() {
	let sample = samples::MERKLE_PROOF_SAMPLE;
	let case = to_test_case(sample);
	let bounded_proofs = BoundedVec::<types::MerkleHash, ConstU32<10>>::try_from(case.1).unwrap();
	let defi_user = true;
	let amount: types::BalanceOf<Test> = 10017332_u64.into();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		let icon_wallet = VALID_ICON_WALLET;
		let ice_address =
			hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

		let mut snapshot = types::SnapshotInfo::default();
		snapshot.done_instant = true;
		snapshot.done_vesting = true;

		pallet_airdrop::IceSnapshotMap::<Test>::insert(&icon_wallet, snapshot);
		let creditor_account = AirdropModule::get_creditor_account();
		pallet_airdrop::ExchangeAccountsMap::<Test>::insert(&icon_wallet, amount);
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

		assert_err!(
			AirdropModule::dispatch_exchange_claim(
				Origin::root(),
				icon_wallet,
				ice_address.clone(),
				amount + 10000,
				defi_user,
				bounded_proofs,
			),
			PalletError::InvalidClaimAmount
		);
	});
}
