use super::prelude::*;
use core::str::FromStr;
use frame_benchmarking::account;
use hex_literal::hex;
use sp_runtime::AccountId32;
use types::{IconVerifiable, SignatureValidationError};



const VALID_ICON_SIGNATURE: types::IconSignature = hex!("628af708622383d60e1d9d95763cf4be64d0bafa8daebb87847f14fde0db40013105586f0c937ddf0e8913251bf01cf8e0ed82e4f631b666453e15e50d69f3b900");
const VALID_MESSAGE: &str = "icx_sendTransaction.data.{method.transfer.params.{wallet.da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c}}.dataType.call.from.hxdd9ecb7d3e441d25e8c4f03cd20a80c502f0c374.nid.0x1.nonce.0x1..timestamp.0x5d56f3231f818.to.cx8f87a4ce573a2e1377545feabac48a960e8092bb.version.0x3";
const VALID_ICON_WALLET: types::IconAddress =
	decode_hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9");
	
const VALID_ICE_ADDRESS: &str = "da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c";
//5GCXPTWkVTJMx4YmnxATMqotWYu1d1uDy2rwWP5PFVi9PmCM
// #[test]
// fn test_pair_signature(){
// 	use sp_core::sr25519::{Pair,Public,Signature};
//     use sp_core::Pair as TraitPair;
// 	let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
// 		let public = pair.public();
// 		assert_eq!(
// 			public,
// 			Public::from_raw(hex!(
// 				"741c08a06f41c596608f6774259bd9043304adfa5d3eea62760bd9be97634d63"
// 			))
// 		);
// 		let message = hex!("2f8c6129d816cf51c374bc7f08c3e63ed156cf78aefb4a6550d97b87997977ee00000000000000000200d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a4500000000000000");
// 		let signature = pair.sign(&message[..]);
// 		let signature_hex=hex::encode(&signature);
// 		//Signature::from_raw(data)

//         assert_eq!(signature_hex,"somesignature");
// 		assert!(Pair::verify(&signature, &message[..], &public));
	
	
	
// }

#[test]
fn test_ice_signature_native(){
	use codec::Decode;
	let mut ice_bytes=hex!("741c08a06f41c596608f6774259bd9043304adfa5d3eea62760bd9be97634d63");
	//let ice_public =Public::from_raw(ice_bytes);
	let signature =hex!("e8dda773f806311db1937816ed5dc9d9051b30fe18e1feb0bbed2dd17cb58960e2787b2c4c725d61d25e08b4fc8be5eac5e3b553e0eaf398fc4e66220e71bb87");
	let message =hex!("2f8c6129d816cf51c374bc7f08c3e63ed156cf78aefb4a6550d97b87997977ee00000000000000000200d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a4500000000000000");
	let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap_or_default();
	let result= AirdropModule::check_signature(signature, &message, ice_bytes,ice_address).unwrap();

    assert!(result);



}

/**
 *iconAddress: "0xb48f3bd3862d4a489fb3c9b761c4cfb20b34a645"
iconSignature: "0x9ee3f663175691ad82f4fbb0cfd0594652e3a034e3b6934b0e4d4a60437ba4043c89d2ffcb7b0af49ed0744ce773612d7ebcdf3a5b035c247706050e0a0033e401"
iconTxObj: "icx_sendTransaction.data.{method.transfer.params.{wallet.0xb6e7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48}}.dataType.call.from.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.version.0x3"
polkadotAddress: "0xb6e7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48"
polkadotSignature: "0x901bda07fb98882a4944f50925b45d041a8a05751a45501eab779416bb55ca5537276dad3c68627a7ddb96956a17ae0d89ca27901a9638ad26426d0e2fbf7e8a"
 */
// from frontend
#[test]
fn test_ice_signature_frontend(){
	use codec::Decode;
	let mut ice_bytes=hex!("b6e7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48");
	//let ice_public =Public::from_raw(ice_bytes);
	let signature =hex!("901bda07fb98882a4944f50925b45d041a8a05751a45501eab779416bb55ca5537276dad3c68627a7ddb96956a17ae0d89ca27901a9638ad26426d0e2fbf7e8a");
	let message =hex!("9ee3f663175691ad82f4fbb0cfd0594652e3a034e3b6934b0e4d4a60437ba4043c89d2ffcb7b0af49ed0744ce773612d7ebcdf3a5b035c247706050e0a0033e401");
	let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap_or_default();
	let result= AirdropModule::check_signature(signature, &message, ice_bytes,ice_address).unwrap();

    assert!(result);



}

// polkadot example
#[test]
fn test_ice_signature_polkadot(){
	use codec::Decode;
	let mut ice_bytes=hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48");
	//let ice_public =Public::from_raw(ice_bytes);
	let signature =hex!("2aeaa98e26062cf65161c68c5cb7aa31ca050cb5bdd07abc80a475d2a2eebc7b7a9c9546fbdff971b29419ddd9982bf4148c81a49df550154e1674a6b58bac84");
	let message ="This is a text message".as_bytes();
	let ice_address =
			<mock::Test as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.unwrap_or_default();
	let result= AirdropModule::check_signature(signature, &message, ice_bytes,ice_address).unwrap();

    assert!(result);



}


#[test]
fn siganture_validation_valid() {
	{
		let icon_signature = VALID_ICON_SIGNATURE;
		let message = VALID_MESSAGE.as_bytes();
		let icon_wallet = VALID_ICON_WALLET;
		let account_id = AccountId32::from_str(VALID_ICE_ADDRESS).unwrap();

		assert_ok!(account_id.verify_with_icon(&icon_wallet, &icon_signature, &message));
	}

	// TODO:
	// add sample of more valid cases
}

#[test]
fn invalid_icon_signature() {
	let icon_wallet = VALID_ICON_WALLET;
	let account_id = AccountId32::from_str(VALID_ICE_ADDRESS).unwrap();

	// When icon address is in expected format but is invalid
	{
		let icon_signature = hex_literal::hex!("3a000000002383d60e1d9d95763cf4be64d0bafa8daebb87847f14fde0db40013105586f0c937ddf0e8913251bf01cf8e0ed82e4f0000000000000000000000000");
		assert_err!(
			account_id.verify_with_icon(&icon_wallet, &icon_signature, VALID_MESSAGE.as_bytes()),
			SignatureValidationError::InvalidIconSignature
		);
	}
}

#[test]
fn invalid_ice_address() {
	let icon_signature = VALID_ICON_SIGNATURE;
	let icon_wallet = VALID_ICON_WALLET;
	let account_id = AccountId32::from_str(VALID_ICE_ADDRESS).unwrap();

	// Valid message but modified ice_address
	{
		let invalid_account_id = AccountId32::from_str(
			"12345123451234512345e13f522693299b9de1b70ff0464caa5d392396a8f76c",
		)
		.unwrap();
		assert_err!(
			invalid_account_id.verify_with_icon(
				&icon_wallet,
				&icon_signature,
				VALID_MESSAGE.as_bytes()
			),
			SignatureValidationError::InvalidIceAddress
		);
	}

	// Valid ice_address but modified message
	{
		let invalid_message = "icx_sendTransaction.data.{method.transfer.params.{wallet.0000000000000000000000000000000000000000000000000000000000000000}}.dataType.call.from.hxdd9ecb7d3e441d25e8c4f03cd20a80c502f0c374.nid.0x1.nonce.0x1..timestamp.0x5d56f3231f818.to.cx8f87a4ce573a2e1377545feabac48a960e8092bb.version.0x3";
		assert_err!(
			account_id.verify_with_icon(&icon_wallet, &icon_signature, invalid_message.as_bytes()),
			SignatureValidationError::InvalidIceAddress
		);
	}
}

#[test]
fn invalid_icon_address() {
	let icon_wallet = samples::ICON_ADDRESS[1];
	let account_id = AccountId32::from_str(VALID_ICE_ADDRESS).unwrap();
	let icon_signature = &VALID_ICON_SIGNATURE;

	assert_err!(
		account_id.verify_with_icon(&icon_wallet, &icon_signature, VALID_MESSAGE.as_bytes()),
		SignatureValidationError::InvalidIconAddress
	);
}

#[test]
fn mock_signature_validation() {
	// It should pass with dummy data, basically anything
	assert_ok!(samples::ACCOUNT_ID[0].verify_with_icon(&[0_u8; 20], &[0u8; 65], &vec![]));
}

#[test]
fn recover_icon_address(){
	let signature= VALID_ICON_SIGNATURE.clone();
	let message =VALID_MESSAGE.as_bytes();
	let icon_address =VALID_ICON_WALLET.to_vec();
	let extracted_address = utils::recover_address(&signature, message).unwrap();
	assert_eq!(icon_address,extracted_address);
}


