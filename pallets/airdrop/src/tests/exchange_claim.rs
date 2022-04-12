use super::prelude::*;
use frame_support::traits::Hooks;
use sp_runtime::DispatchError;
const VALID_ICON_WALLET: types::IconAddress =
	decode_hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9");

#[test]
fn claim_success() {
	use codec::Decode;

	let server_data = samples::SERVER_DATA[0];
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		let icon_wallet = VALID_ICON_WALLET;
		let ice_bytes =
			hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

		let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap_or_default();

		let creditor_account = AirdropModule::get_creditor_account();
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
			server_data.clone()
		));
	});
}

#[test]
fn insufficient_balance() {
	use codec::Decode;

	let server_data = samples::SERVER_DATA[0];
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		let icon_wallet = VALID_ICON_WALLET;
		let ice_bytes =
			hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

		let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap_or_default();

		let creditor_account = AirdropModule::get_creditor_account();
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_u32.into(),
			10_u32.into(),
		)
		.unwrap();

		assert_noop!(
			AirdropModule::dispatch_exchange_claim(
				Origin::root(),
				icon_wallet,
				ice_address.clone(),
				server_data.clone()
			),
			PalletError::InsufficientCreditorBalance
		);
	});
}
#[test]
fn already_claimed() {
	use codec::Decode;

	let server_data = samples::SERVER_DATA[0];
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		let icon_wallet = VALID_ICON_WALLET;
		let ice_bytes =
			hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");

		let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap_or_default();

		pallet_airdrop::IceSnapshotMap::<Test>::insert(
			&icon_wallet,
			types::SnapshotInfo::default(),
		);

		assert_noop!(
			AirdropModule::dispatch_exchange_claim(
				Origin::root(),
				icon_wallet,
				ice_address.clone(),
				server_data.clone()
			),
			PalletError::ClaimAlreadyMade
		);
	});
}
