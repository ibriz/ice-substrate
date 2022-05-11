use super::{prelude::*};
use crate::tests::UserClaimTestCase;

#[test]
#[cfg(not(feature = "no-vesting"))]
fn claim_success() {
	let ofw_account = samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));
		let mut case  =UserClaimTestCase::default();
		case.amount=12_017_332_u64;
		

		let creditor_account = AirdropModule::get_creditor_account();
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

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
		let ice_account=AirdropModule::to_account_id(case.ice_address.clone()).unwrap();
		let claim_balance =
			<Test as pallet_airdrop::Config>::Currency::usable_balance(ice_account);
		assert_eq!(claim_balance, 6761333);
		let snapshot =<pallet_airdrop::IceSnapshotMap<Test>>::get(&case.icon_address).unwrap();
		assert_eq!(snapshot.done_vesting,true);
		assert_eq!(snapshot.done_instant,true);
	});
}

#[test]
#[cfg(feature = "no-vesting")]
fn claim_success() {
	let ofw_account = samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));

		let mut case  =UserClaimTestCase::default();
		case.amount=12_017_332_u64;

		let creditor_account = AirdropModule::get_creditor_account();
		<Test as pallet_airdrop::Config>::Currency::set_balance(
			mock::Origin::root(),
			creditor_account,
			10_000_0000_u32.into(),
			10_000_00_u32.into(),
		)
		.unwrap();

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
		let ice_account=AirdropModule::to_account_id(case.ice_address.clone()).unwrap();
		let claim_balance =
			<Test as pallet_airdrop::Config>::Currency::usable_balance(&ice_account);
		assert_eq!(claim_balance, 12_017_332);
		let snapshot =<pallet_airdrop::IceSnapshotMap<Test>>::get(&case.icon_address).unwrap();
		assert_eq!(snapshot.done_vesting,false);
		assert_eq!(snapshot.done_instant,true);
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

		let mut case =UserClaimTestCase::default();
		case.amount=10017332_u64;
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
		let mut case =UserClaimTestCase::default();
		case.amount=10017332_u64;

		let mut snapshot = types::SnapshotInfo::default();
		snapshot.done_instant = true;
		snapshot.done_vesting = true;

		pallet_airdrop::IceSnapshotMap::<Test>::insert(&case.icon_address, snapshot);
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

		case.message = "icx_sendTransaction.data.{method.transfer.params.{wallet.eee7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48}}.dataType.call.from.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.version.0x3".as_bytes().to_vec();
		
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
		let mut case =UserClaimTestCase::default();
		case.ice_signature=[0u8;64];

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
		let mut case =UserClaimTestCase::default();
		case.icon_signature=[0u8;65];

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

