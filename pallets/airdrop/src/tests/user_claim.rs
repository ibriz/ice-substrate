use super::prelude::*;
use codec::Decode;
use frame_support::traits::Hooks;
use sp_runtime::DispatchError;
const VALID_ICON_SIGNATURE:types::IconSignature= decode_hex!("628af708622383d60e1d9d95763cf4be64d0bafa8daebb87847f14fde0db40013105586f0c937ddf0e8913251bf01cf8e0ed82e4f631b666453e15e50d69f3b900");
const VALID_MESSAGE: &str = "icx_sendTransaction.data.{method.transfer.params.{wallet.da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c}}.dataType.call.from.hxdd9ecb7d3e441d25e8c4f03cd20a80c502f0c374.nid.0x1.nonce.0x1..timestamp.0x5d56f3231f818.to.cx8f87a4ce573a2e1377545feabac48a960e8092bb.version.0x3";
const VALID_ICON_WALLET: types::IconAddress =
	decode_hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9");

#[test]
fn claim_success() {
	let server_data = samples::SERVER_DATA[0];
	let (mut test_ext, offchain_state, pool_state, ocw_pub) = offchain_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_offchain_account(
			Origin::root(),
			ocw_pub.into_account()
		));

		let icon_signature = VALID_ICON_SIGNATURE.clone();
		let message = VALID_MESSAGE.as_bytes();
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

		assert_ok!(AirdropModule::dispatch_user_claim(
			Origin::signed(AirdropModule::get_offchain_account().unwrap()),
			icon_wallet,
			ice_address.clone(),
			message.to_vec(),
			icon_signature,
			server_data.clone()
		));
	});
}

#[test]
fn insufficient_balance() {
	let server_data = samples::SERVER_DATA[0];
	let (mut test_ext, offchain_state, pool_state, ocw_pub) = offchain_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_offchain_account(
			Origin::root(),
			ocw_pub.into_account()
		));

		let icon_signature = VALID_ICON_SIGNATURE.clone();
		let message = VALID_MESSAGE.as_bytes();
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

		assert_err!(
			AirdropModule::dispatch_user_claim(
				Origin::signed(AirdropModule::get_offchain_account().unwrap()),
				icon_wallet,
				ice_address.clone(),
				message.to_vec(),
				icon_signature,
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
	let (mut test_ext, offchain_state, pool_state, ocw_pub) = offchain_test_ext();
	test_ext.execute_with(|| {
        assert_ok!(AirdropModule::set_offchain_account(
			Origin::root(),
			ocw_pub.into_account()
		));
		let icon_wallet = VALID_ICON_WALLET;
		let ice_bytes =
			hex_literal::hex!("da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c");
		let icon_signature = VALID_ICON_SIGNATURE.clone();
		let message = VALID_MESSAGE.as_bytes();
		let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap_or_default();
		let mut snapshot = types::SnapshotInfo::default();
		snapshot.done_instant= true;
		snapshot.done_vesting= true;

		pallet_airdrop::IceSnapshotMap::<Test>::insert(
			&icon_wallet,
			snapshot,
		);
		let creditor_account = AirdropModule::get_creditor_account();
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

		assert_noop!(
			AirdropModule::dispatch_user_claim(
				Origin::signed(AirdropModule::get_offchain_account().unwrap()),
				icon_wallet,
				ice_address.clone(),
				message.to_vec(),
				icon_signature,
				server_data.clone()
			),
			PalletError::ClaimAlreadyMade
		);
	});
}
