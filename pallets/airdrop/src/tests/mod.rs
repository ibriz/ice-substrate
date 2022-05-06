use crate::{mock, types::{MerkleHash, MerkleProofs}};
mod signature_validation;
mod transfer;
mod utility_functions;
mod exchange_claim;
mod user_claim;
mod merkle_tests;
pub mod prelude {
	pub use super::{
		assert_tx_call, credit_creditor, get_last_event, minimal_test_ext, not_offchain_account,
		put_response, run_to_block, samples,
	};
	pub use crate as pallet_airdrop;
	pub use crate::tests;
	pub use frame_support::{
		assert_err, assert_err_ignore_postinfo, assert_err_with_weight, assert_noop, assert_ok,
		assert_storage_noop,
	};
	pub use hex_literal::hex as decode_hex;
	pub use pallet_airdrop::mock::{self, AirdropModule, Origin, Test};
	pub use pallet_airdrop::{types, utils};
	pub use sp_core::bytes;
	pub use sp_runtime::traits::{Bounded, IdentifyAccount, Saturating};

	pub type PalletError = pallet_airdrop::Error<Test>;
	pub type PalletEvent = pallet_airdrop::Event<Test>;
	pub type PalletCall = pallet_airdrop::Call<Test>;
	pub type BalanceError = pallet_balances::Error<Test>;
}

use mock::System;
use prelude::*;

pub struct SignatureTestCase {
	pub icon_address:[u8;20],
	pub icon_signature:[u8;65],
	pub ice_address:[u8;32],
	pub ice_signature:[u8;64],
	pub message:Vec<u8>

}

pub mod samples {

use super::decode_hex;
	use super::types::{IconAddress, ServerResponse};
	use sp_core::sr25519;

	pub const ACCOUNT_ID: &[sr25519::Public] = &[
		sr25519::Public([1; 32]),
		sr25519::Public([2; 32]),
		sr25519::Public([3; 32]),
		sr25519::Public([4; 32]),
		sr25519::Public([5; 32]),
	];

	pub const SERVER_DATA: &[ServerResponse] = &[
		ServerResponse {
			omm: 1234443,
			amount: 345323,
			stake: 8437566,
			defi_user: true,
		},
		ServerResponse {
			omm: 8548467,
			amount: 928333,
			stake: 298329,
			defi_user: false,
		},
	];

	pub const ICON_ADDRESS: &[IconAddress] = &[
		decode_hex!("ee1448f0867b90e6589289a4b9c06ac4516a75a9"),
		decode_hex!("ee33286f367b90e6589289a4b987a6c4516a753a"),
		decode_hex!("ee12463586abb90e6589289a4b9c06ac4516a7ba"),
		decode_hex!("ee02363546bcc50e643910104321c0623451a65a"),
	];

}

/// Dummy implementation for IconVerififable trait for test AccountId
/// This implementation always passes so should not be dependent upon
impl types::IconVerifiable for sp_core::sr25519::Public {
	fn verify_with_icon(
		&self,
		_icon_wallet: &types::IconAddress,
		_icon_signature: &types::IconSignature,
		_message: &[u8],
	) -> Result<(), types::SignatureValidationError> {
		Ok(())
	}
}

// Build genesis storage according to the mock runtime.
pub fn minimal_test_ext() -> sp_io::TestExternalities {
		use hex_literal::hex;
		use codec::Decode;
		use frame_support::traits::GenesisBuild;
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		let account_hex=hex!["d893ef775b5689473b2e9fa32c1f15c72a7c4c86f05f03ee32b8aca6ce61b92c"];
		let account_id = types::AccountIdOf::<Test>::decode(&mut &account_hex[..]).unwrap();
		pallet_airdrop::GenesisConfig::<Test> { 
			creditor_account: Some(account_id), 
			exchange_accounts:vec![] 
		}
			.assimilate_storage(&mut t)
			.unwrap();
		t.into()
}

// Return the same address if it is not sudo
pub fn not_offchain_account(account: types::AccountIdOf<Test>) -> types::AccountIdOf<Test> {
	if account != AirdropModule::get_airdrop_server_account().unwrap_or_default() {
		account
	} else {
		panic!("This address must not be same as defined in offchian worker. Change test value.");
	}
}

pub fn run_to_block(n: types::BlockNumberOf<Test>) {
	use frame_support::traits::Hooks;

	while System::block_number() < n {
		if System::block_number() > 1 {
			AirdropModule::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		AirdropModule::on_initialize(System::block_number());
		//<Test as pallet_airdrop::Config>::VestingModule::on_initialize(System::block_number());
	}
}

use sp_core::offchain::testing;
pub fn put_response(
	state: &mut testing::OffchainState,
	icon_address: &types::IconAddress,
	expected_response: &str,
) {
	let uri = String::from_utf8(
		mock::FetchIconEndpoint::get()
			.as_bytes()
			.iter()
			.chain(bytes::to_hex(icon_address, false).as_bytes().iter())
			.cloned()
			.collect::<Vec<u8>>(),
	)
	.unwrap();

	let response = if expected_response.is_empty() {
		None
	} else {
		Some(expected_response.to_string().as_bytes().to_vec())
	};

	state.expect_request(testing::PendingRequest {
		method: "GET".to_string(),
		uri,
		response,
		sent: true,
		..Default::default()
	});
}

pub fn get_last_event() -> Option<<Test as frame_system::Config>::Event> {
	<frame_system::Pallet<Test>>::events()
		.pop()
		.map(|v| v.event)
}

pub fn assert_tx_call(expected_call: &[&PalletCall], pool_state: &testing::PoolState) {
	use codec::Encode;

	let all_calls_in_pool = &pool_state.transactions;
	let expected_call_encoded = expected_call
		.iter()
		.map(|call| call.encode())
		.collect::<Vec<_>>();
	let all_calls_in_pool = all_calls_in_pool
		.iter()
		.enumerate()
		.map(|(index, call)| &call[call.len() - expected_call_encoded[index].len()..])
		.collect::<Vec<_>>();

	assert_eq!(expected_call_encoded, all_calls_in_pool);
}

pub fn credit_creditor(balance: u64) {
	let creditor_account = AirdropModule::get_creditor_account();
	let deposit_res = <Test as pallet_airdrop::Config>::Currency::set_balance(
		mock::Origin::root(),
		creditor_account,
		balance.into(),
		0u32.into(),
	);

	assert_ok!(deposit_res);
	assert_eq!(
		<Test as pallet_airdrop::Config>::Currency::free_balance(&creditor_account),
		balance.into()
	);
}

pub fn to_test_case(sample:(String,Vec<String>))->(MerkleHash,Vec<MerkleHash>){
	let mut  hash_bytes =[0u8; 32];
	hex::decode_to_slice(sample.0, &mut hash_bytes as &mut [u8]).unwrap();
	let proofs =sample.1.iter().map(|p|{
		let mut bytes: [u8; 32] = [0u8; 32];
				hex::decode_to_slice(p, &mut bytes as &mut [u8]).unwrap();
				bytes

	}).collect::<Vec<MerkleHash>>();
	
	(hash_bytes,proofs)
}

pub fn get_merkle_proof_sample()->(String,Vec<String>){
	let sample=(
		"7fe522d63ebcabfa052eec3647366138c23c9870995f4af94d9b22b8c5923f49".to_owned(),
		vec![
			"813340daefd7f1ca705faf8318cf6455632259d113c06e97b70eeeccd43519a9".to_owned(),
			"409519ab7129397bdc895e4da05871c9725697a5e092addf2fe90f6e795feb8f".to_owned(),
			"38055bb872670c69ac3461707f8c0b4b8e436eecfc84cfd80db30db3030c489a".to_owned(),
		],
	);
	return sample;
}