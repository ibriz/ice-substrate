use super::prelude::*;
use core::str::FromStr;
use hex_literal::hex;
use sp_runtime::AccountId32;
use types::{IconVerifiable, SignatureValidationError};

const VALID_ICON_SIGNATURE: types::IconSignature = hex!("628af708622383d60e1d9d95763cf4be64d0bafa8daebb87847f14fde0db40013105586f0c937ddf0e8913251bf01cf8e0ed82e4f631b666453e15e50d69f3b900");
const VALID_MESSAGE: &str = "icx_sendTransaction.data.{method.transfer.params.{wallet.da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c}}.dataType.call.from.hxdd9ecb7d3e441d25e8c4f03cd20a80c502f0c374.nid.0x1.nonce.0x1..timestamp.0x5d56f3231f818.to.cx8f87a4ce573a2e1377545feabac48a960e8092bb.version.0x3";
const VALID_ICON_WALLET: types::IconAddress =
	decode_hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9");
	
const VALID_ICE_ADDRESS: &str = "da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c";

const ICE_EVM_SIGNATURE:[u8;65]= hex!("08e84ee9507730f4bd7eee71b15814e8a4f4cb835d939ac70a68453d12793416127fb61437741a622fdb5685959e5885ef702281f3f8058e357618e7ba8bd2091c");
const ICE_EVM_PAYLOAD:[u8;65]=   hex!("458386f5e63c05a9a762279c14b1d7de407e5f15a0049c31710be09422d80d6c22ee498c6ef1753f5b08af34c66f624795ba04a2cdb39ed2a174756c1e7defb300");
const ICE_EVM_ADDRESS:[u8;20]= hex!("027e99362a6b2EdCcC9341b4b49f639e4F26D34F");


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

#[test]
fn recover_ice_evm_address(){
	let signature= ICE_EVM_SIGNATURE.clone();
	let message =&ICE_EVM_PAYLOAD;
	let icon_address =ICE_EVM_ADDRESS.to_vec();
	let extracted_address = utils::eth_recover(&signature, message,&[][..]).unwrap();

	assert_eq!(icon_address,extracted_address);



}


