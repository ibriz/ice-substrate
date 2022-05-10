use super::prelude::*;
use core::str::FromStr;
use frame_benchmarking::account;
use hex_literal::hex;
use sp_runtime::AccountId32;
use types::{IconVerifiable, SignatureValidationError};

const VALID_ICON_SIGNATURE: types::IconSignature = hex!("9ee3f663175691ad82f4fbb0cfd0594652e3a034e3b6934b0e4d4a60437ba4043c89d2ffcb7b0af49ed0744ce773612d7ebcdf3a5b035c247706050e0a0033e401");
const VALID_MESSAGE: &str = "icx_sendTransaction.data.{method.transfer.params.{wallet.b6e7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48}}.dataType.call.from.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hxb48f3bd3862d4a489fb3c9b761c4cfb20b34a645.version.0x3";
const VALID_ICON_WALLET: types::IconAddress =
	decode_hex!("b48f3bd3862d4a489fb3c9b761c4cfb20b34a645");
const VALID_ICE_ADDRESS: &str = "b6e7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48";
const VALID_ICE_SIGNATURE : &str="901bda07fb98882a4944f50925b45d041a8a05751a45501eab779416bb55ca5537276dad3c68627a7ddb96956a17ae0d89ca27901a9638ad26426d0e2fbf7e8a";

#[test]
fn test_ice_signature_native() {
	let ice_bytes = hex!("741c08a06f41c596608f6774259bd9043304adfa5d3eea62760bd9be97634d63");
	let signature =hex!("e8dda773f806311db1937816ed5dc9d9051b30fe18e1feb0bbed2dd17cb58960e2787b2c4c725d61d25e08b4fc8be5eac5e3b553e0eaf398fc4e66220e71bb87");
	let message =hex!("2f8c6129d816cf51c374bc7f08c3e63ed156cf78aefb4a6550d97b87997977ee00000000000000000200d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a4500000000000000");
	let result = AirdropModule::check_signature(&signature, &message, ice_bytes);

	assert!(result);
}
#[test]
fn test_ice_signature_frontend_plain_message() {
	let ice_bytes = hex!("14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f");
	let signature =hex!("42b054d71be08205377b8f9fa1e96fbb45bfe8889d5cc8019e41ff6ea6364525669092b385920b38d7d289f312e63d9ea4d036e2989909926b5127417784eb83");
	let message = "Message to Sign".as_bytes();
	let wrapped_message = utils::wrap_bytes(message);
	let result = AirdropModule::check_signature(&signature, &wrapped_message, ice_bytes);

	assert!(result);
}

#[test]
fn test_ice_signature_frontend_icon_signature() {
	use codec::Decode;
	let ice_bytes = hex!("14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f");

	let signature =hex!("62ff224a8401451ffd32e8d56bef2253ecebdf9d5fa825ccd2de823ccebad34cdf18ea924273cd1e735ca1a0ec8a4b2a61333bc0ec8d0a1f6ff08d8cf25a9080");
	let message =  hex!("11f7dc15685555af583228f14e6f5766cf339d3c38389ce022f10a468296dde864df99d9056b7ee7116a290713ba38c7ca7fcf161fc8137a039445d0701c4dbb00");
	let wrapped_message = utils::wrap_bytes(&message);
	let result = AirdropModule::check_signature(&signature, &wrapped_message, ice_bytes);

	assert!(result);

	assert!(result);
}

#[test]
fn test_ice_signature_frontend_icon_signature_2() {
	let icon_address = hex!("b48f3bd3862d4a489fb3c9b761c4cfb20b34a645");
	let ice_bytes = hex!("b6e7a79d04e11a2dd43399f677878522523327cae2691b6cd1eb972b5a88eb48");

	let ice_signature =hex!("901bda07fb98882a4944f50925b45d041a8a05751a45501eab779416bb55ca5537276dad3c68627a7ddb96956a17ae0d89ca27901a9638ad26426d0e2fbf7e8a");
	let icon_signature =  hex!("9ee3f663175691ad82f4fbb0cfd0594652e3a034e3b6934b0e4d4a60437ba4043c89d2ffcb7b0af49ed0744ce773612d7ebcdf3a5b035c247706050e0a0033e401");
	let wrapped_message = utils::wrap_bytes(&icon_signature);

	let result = AirdropModule::check_signature(&ice_signature, &wrapped_message, ice_bytes);

	assert!(result);
}

#[test]
fn test_ice_signature_polkadot() {
	let ice_bytes = hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48");
	let signature =hex!("2aeaa98e26062cf65161c68c5cb7aa31ca050cb5bdd07abc80a475d2a2eebc7b7a9c9546fbdff971b29419ddd9982bf4148c81a49df550154e1674a6b58bac84");
	let message = "This is a text message".as_bytes();
	let result = AirdropModule::check_signature(&signature, &message, ice_bytes);

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
fn recover_icon_address() {
	let signature = VALID_ICON_SIGNATURE.clone();
	let message = VALID_MESSAGE.as_bytes();
	let icon_address = VALID_ICON_WALLET.to_vec();
	let extracted_address = utils::recover_address(&signature, message).unwrap();
	assert_eq!(icon_address, extracted_address);
}
