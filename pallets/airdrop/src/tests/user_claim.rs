use crate::{types::{MerkleProofs, MerkleHash}, tests::get_merkle_proof_sample};

use super::{prelude::*, to_test_case};
use codec::Decode;
use frame_support::{traits::ConstU32, BoundedVec};
const VALID_ICON_SIGNATURE:types::IconSignature= decode_hex!("9ee3f663175691ad82f4fbb0cfd0594652e3a034e3b6934b0e4d4a60437ba4043c89d2ffcb7b0af49ed0744ce773612d7ebcdf3a5b035c247706050e0a0033e401");
const VALID_MESSAGE: &str = "icx_sendTransaction.data.{method.transfer.params.{wallet.b6e7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48}}.dataType.call.from.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.version.0x3";
const VALID_ICON_WALLET: types::IconAddress =decode_hex!("b48f3bd3862d4a489fb3c9b761c4cfb20b34a645");
const VALID_ICE_ADDRESS: [u8;32] = decode_hex!("b6e7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48");
const VALID_ICE_SIGNATURE : [u8;64] =decode_hex!("901bda07fb98882a4944f50925b45d041a8a05751a45501eab779416bb55ca5537276dad3c68627a7ddb96956a17ae0d89ca27901a9638ad26426d0e2fbf7e8a");



#[test]
fn claim_success() {
    let defi_user=true;
	let sample =get_merkle_proof_sample();
    let case =to_test_case(sample);
	let bounded_proofs=BoundedVec::<types::MerkleHash,ConstU32<10>>::try_from(case.1).unwrap();
    let amount=10017332_u64;
	let ofw_account= samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));

		let icon_signature = VALID_ICON_SIGNATURE.clone();
		let message = VALID_MESSAGE.as_bytes();
		let icon_wallet = VALID_ICON_WALLET;
		let ice_bytes = VALID_ICE_ADDRESS.clone();
		let ice_signature =VALID_ICE_SIGNATURE.clone();

		let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap();

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
			icon_wallet,
			ice_address.clone(),
			message.to_vec(),
			icon_signature,
			ice_signature,
			amount,
            defi_user,
			bounded_proofs,
		));
	});
}

#[test]
fn insufficient_balance() {
	let defi_user=true;
    let amount=10017332_u64;
	let sample =get_merkle_proof_sample();
    let case =to_test_case(sample);
	let bounded_proofs=BoundedVec::<types::MerkleHash,ConstU32<10>>::try_from(case.1).unwrap();
	let ofw_account= samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
		assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));

		let icon_signature = VALID_ICON_SIGNATURE.clone();
		let message = VALID_MESSAGE.as_bytes();
		let icon_wallet = VALID_ICON_WALLET;
		let ice_bytes = VALID_ICE_ADDRESS.clone();
		let ice_signature =VALID_ICE_SIGNATURE.clone();

		let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap();

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
				icon_wallet,
				ice_address.clone(),
				message.to_vec(),
				icon_signature,
				ice_signature,
				amount,
                defi_user,
			    bounded_proofs
			),
			PalletError::InsufficientCreditorBalance
		);
	});
}

#[test]
fn already_claimed() {
	use codec::Decode;
	let sample =get_merkle_proof_sample();
    let case =to_test_case(sample);
	let bounded_proofs=BoundedVec::<types::MerkleHash,ConstU32<10>>::try_from(case.1).unwrap();
	let defi_user=true;
    let amount=10017332_u64;
	let ofw_account= samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
        assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));
		let icon_wallet = VALID_ICON_WALLET;
		let icon_signature = VALID_ICON_SIGNATURE.clone();

		let ice_bytes = VALID_ICE_ADDRESS.clone();
		let ice_signature =VALID_ICE_SIGNATURE.clone();

		let message = VALID_MESSAGE.as_bytes();
		let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap();
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

		assert_err!(
			AirdropModule::dispatch_user_claim(
				Origin::signed(AirdropModule::get_airdrop_server_account().unwrap()),
				icon_wallet,
				ice_address.clone(),
				message.to_vec(),
				icon_signature,
				ice_signature,
				amount,
                defi_user,
			    bounded_proofs,
			),
			PalletError::ClaimAlreadyMade
		);
	});
}

#[test]
fn invalid_payload() {
	use codec::Decode;
	let sample =get_merkle_proof_sample();
    let case =to_test_case(sample);
	let bounded_proofs=BoundedVec::<types::MerkleHash,ConstU32<10>>::try_from(case.1).unwrap();
	let defi_user=true;
    let amount=10017332_u64;
	let ofw_account= samples::ACCOUNT_ID[0].into_account();
	let mut test_ext = minimal_test_ext();
	test_ext.execute_with(|| {
        assert_ok!(AirdropModule::set_airdrop_server_account(
			Origin::root(),
			ofw_account
		));
		let icon_wallet = VALID_ICON_WALLET;
		let icon_signature = VALID_ICON_SIGNATURE.clone();

		let ice_bytes = VALID_ICE_ADDRESS.clone();
		let ice_signature =VALID_ICE_SIGNATURE.clone();

		let message = "icx_sendTransaction.data.{method.transfer.params.{wallet.eee7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48}}.dataType.call.from.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.version.0x3".as_bytes();
		let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap();
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
				icon_wallet,
				ice_address.clone(),
				message.to_vec(),
				icon_signature,
				ice_signature,
				amount,
                defi_user,
			    bounded_proofs,
			),
			PalletError::InvalidMessagePayload
		);
	});
}



