use crate as airdrop;
use airdrop::types;
use sp_core::H160;
use sp_runtime::{
	traits::{BlakeTwo256, Bounded, CheckedDiv, Convert, Saturating},
	AccountId32,
};

/// Reuturns an optional vesting schedule which when applied release given amount
/// which will be complete in given block. If
/// Also return amount which is remaineder if amount can't be perfectly divided
/// in per block basis
pub fn new_vesting_with_deadline<T, const VESTING_APPLICABLE_FROM: u32>(
	amount: types::VestingBalanceOf<T>,
	ends_in: types::BlockNumberOf<T>,
) -> (Option<types::VestingInfoOf<T>>, types::VestingBalanceOf<T>)
where
	T: pallet_vesting::Config,
{
	const MIN_AMOUNT_PER_BLOCK: u32 = 1u32;

	type BlockToBalance<T> = <T as pallet_vesting::Config>::BlockNumberToBalance;
	let mut vesting = None;

	let ends_in_as_balance = BlockToBalance::<T>::convert(ends_in);
	let transfer_over = ends_in_as_balance.saturating_sub(VESTING_APPLICABLE_FROM.into());

	let idol_transfer_multiple = transfer_over * MIN_AMOUNT_PER_BLOCK.into();

	let remainding_amount = amount % idol_transfer_multiple;
	let primary_transfer_amount = amount.saturating_sub(remainding_amount);

	let per_block = primary_transfer_amount
		.checked_div(&idol_transfer_multiple)
		.unwrap_or(Bounded::min_value());

	if per_block > Bounded::min_value() {
		vesting = Some(types::VestingInfoOf::<T>::new(
			primary_transfer_amount,
			per_block,
			VESTING_APPLICABLE_FROM.into(),
		));
	}

	(vesting, remainding_amount)
}

// // Return the iterator that resolves to iterator which in turn resolves to item
// // stored in PendingClaims of On-chain storage
// impl<T: Config> core::iter::Iterator for types::PendingClaimsOf<T> {
// 	// This iterator returns a block number and an iterator to entiries
// 	// in PendingClaims under same block number
// 	type Item = (
// 		types::BlockNumberOf<T>,
// 		frame_support::storage::KeyPrefixIterator<types::IconAddress>,
// 	);

// 	fn next(&mut self) -> Option<Self::Item> {
// 		// Take the block to process
// 		let this_block = self.range.start;
// 		// Increment start by one
// 		self.range.start = this_block + 1_u32.into();

// 		// Check if range is valid
// 		if self.range.start > self.range.end {
// 			return None;
// 		}

// 		// Get the actual iterator result
// 		let this_block_iter = <crate::PendingClaims<T>>::iter_key_prefix(this_block);

// 		Some((this_block, this_block_iter))
// 	}
// }

// impl<T: Config> types::PendingClaimsOf<T> {
// 	pub fn new(range: core::ops::Range<types::BlockNumberOf<T>>) -> Self {
// 		types::PendingClaimsOf::<T> { range }
// 	}
// }

/// Returns total sum of amount returned from server
pub fn get_response_sum(
	server_response: &types::ServerResponse,
) -> Result<types::ServerBalance, sp_runtime::ArithmeticError> {
	use sp_runtime::ArithmeticError::Overflow;

	server_response
		.amount
		.checked_add(server_response.stake)
		.ok_or(Overflow)?
		.checked_add(server_response.omm)
		.ok_or(Overflow)
}

/// Implement IconVerifiable for Anything that can be decoded into Vec<u8>
/// However note that, if AccountId's type is changed. This implementation might
/// also need modification
impl types::IconVerifiable for sp_runtime::AccountId32 {
	/// Function to make sure that icon_address, ice_address and message are in sync
	/// On a high level, it does so by checking for these two verification
	/// * Make sure that icon_signature is equal to or greater than 65
	/// * Make sure that the ice_address encoded in the message and passed
	///    in this function (i.e dispatchable from where this function is called)
	///    are same
	/// * Make sure that this message is signed by the same icon_address
	///    that is being passed to this function (i.e caller for this function)
	///
	/// @return:
	/// verbose error type which point exactly where the process failed
	///
	/// @parameter:
	/// * ice_address: ss58 encoded bytes of origin of parent dispatchable
	/// * icon_address: icon_address
	/// * icon_signature: icon signature
	/// * message: raw message
	fn verify_with_icon(
		&self,
		icon_wallet: &types::IconAddress,
		icon_signature: &types::IconSignature,
		message: &[u8],
	) -> Result<(), types::SignatureValidationError> {
		use codec::Encode;
		use fp_evm::LinearCostPrecompile;
		use frame_support::ensure;
		use pallet_evm_precompile_sha3fips::Sha3FIPS256;
		use pallet_evm_precompile_simple::ECRecoverPublicKey;
		use types::SignatureValidationError;

		const COST: u64 = 1;
		const PADDING_FOR_V: [u8; 31] = [0; 31];

		let ice_address = hex::encode(self.encode());
		let ice_address = ice_address.as_bytes();

		/* =======================================
				Validate the icon_signature length
		*/
		ensure!(
			icon_signature.len() == 65,
			SignatureValidationError::InvalidIconSignature
		);
		// === verified the length of icon_signature

		/* ======================================================
			Verify that the message constains the same ice_address
			as being passed to this function
		*/
		let extracted_ice_address = {
			// TODO:
			// make sure that message will always be in expected format
			const PREFIX_LEN: usize =
				b"ice_sendTransaction.data.{method.transfer.params.{wallet.".len();
			let address_len = ice_address.len();
			&message[PREFIX_LEN..PREFIX_LEN + address_len]
		};

		ensure!(
			&ice_address == &extracted_ice_address,
			SignatureValidationError::InvalidIceAddress
		);
		// ==== Verfiied that ice_address in encoded message
		// and recived in function parameterare same

		/* ================================================
			verify thet this message is being signed by same
			icon_address as passed in this function
		*/
		let (_exit_status, message_hash) = Sha3FIPS256::execute(&message, COST)
			.map_err(|_| SignatureValidationError::Sha3Execution)?;
		let formatted_icon_signature = {
			let sig_r = &icon_signature[..32];
			let sig_s = &icon_signature[32..64];
			let sig_v = &icon_signature[64..];

			// Sig final is in the format of:
			// object hash + 31 byte padding + 1 byte v + 32 byte r + 32 byte s
			message_hash
				.iter()
				.chain(&PADDING_FOR_V)
				.chain(sig_v)
				.chain(sig_r)
				.chain(sig_s)
				.cloned()
				.collect::<sp_std::vec::Vec<u8>>()
		};

		let (_exit_status, icon_pub_key) =
			ECRecoverPublicKey::execute(&formatted_icon_signature, COST)
				.map_err(|_| SignatureValidationError::InvalidIconSignature)?;

		let (_exit_status, computed_icon_address) = Sha3FIPS256::execute(&icon_pub_key, COST)
			.map_err(|_| SignatureValidationError::Sha3Execution)?;

		ensure!(
			&computed_icon_address[computed_icon_address.len() - 20..] == icon_wallet.as_slice(),
			SignatureValidationError::InvalidIconAddress
		);
		// ===== It is now verified that the message is signed by same icon address
		// as passed in this function

		Ok(())
	}
}

/// Try to decode bytes returned from server to
/// ServerResponse if response is valid
/// ServerError if server returned some known error
/// If not return InvalidResponse
pub fn unpack_server_response(
	response_bytes: &[u8],
) -> Result<types::ServerResponse, types::ClaimError> {
	let deserialize_response_res = serde_json::from_slice::<types::ServerResponse>(response_bytes);

	match deserialize_response_res {
		Ok(response) => Ok(response),

		Err(_) => {
			let deserialize_error_res =
				serde_json::from_slice::<types::ServerError>(response_bytes);

			match deserialize_error_res {
				Ok(server_error) => Err(types::ClaimError::ServerError(server_error)),
				Err(_) => Err(types::ClaimError::InvalidResponse),
			}
		}
	}
}

pub fn recover_address(
	signature: &[u8],
	payload: &[u8],
) -> Result<Vec<u8>, types::SignatureValidationError> {
	use fp_evm::LinearCostPrecompile;
	use pallet_evm_precompile_sha3fips::Sha3FIPS256;
	use pallet_evm_precompile_simple::ECRecoverPublicKey;
	use types::SignatureValidationError;

	const COST: u64 = 1;
	const PADDING_FOR_V: [u8; 31] = [0; 31];

	let (_exit_status, message_hash) =
		Sha3FIPS256::execute(payload, COST).map_err(|_| SignatureValidationError::Sha3Execution)?;
	let formatted_signature = {
		let sig_r = &signature[..32];
		let sig_s = &signature[32..64];
		let sig_v = &signature[64..];

		// Sig final is in the format of:
		// object hash + 31 byte padding + 1 byte v + 32 byte r + 32 byte s
		message_hash
			.iter()
			.chain(&PADDING_FOR_V)
			.chain(sig_v)
			.chain(sig_r)
			.chain(sig_s)
			.cloned()
			.collect::<sp_std::vec::Vec<u8>>()
	};

	let (_exit_status, recovered_pub_key) = ECRecoverPublicKey::execute(&formatted_signature, COST)
		.map_err(|_| SignatureValidationError::InvalidIconSignature)?;

	let (_exit_status, computed_address) = Sha3FIPS256::execute(&recovered_pub_key, COST)
		.map_err(|_| SignatureValidationError::Sha3Execution)?;
	let address = computed_address[computed_address.len() - 20..].to_vec();

	Ok(address)
}

pub fn into_account_id(address: H160) -> AccountId32 {
	let mut data = [0u8; 24];
	data[0..4].copy_from_slice(b"evm:");
	data[4..24].copy_from_slice(&address[..]);
	let hash = <BlakeTwo256 as sp_runtime::traits::Hash>::hash(&data);
	AccountId32::from(Into::<[u8; 32]>::into(hash))
}

fn hash_personal(what: &[u8], extra: &[u8]) -> Vec<u8> {
	let mut v = b"\x19Ethereum Signed Message:\n65".to_vec();
	v.extend_from_slice(what);
	v.extend_from_slice(extra);
	v
}

pub fn eth_recover(s: &[u8; 65], what: &[u8], extra: &[u8]) -> Option<Vec<u8>> {
	use sp_io::{
		crypto::secp256k1_ecdsa_recover, crypto::secp256k1_ecdsa_recover_compressed,
		hashing::keccak_256,
	};
	let msg = keccak_256(&hash_personal(&what,&extra));
	let mut res = [0; 20];
	// res.copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s, &msg).ok()?[..])[12..]);
	// let res =recover_address(s,&msg).ok()?;
	let res = secp256k1_ecdsa_recover_compressed(s, &msg).ok().unwrap();
	Some(res.to_vec())
}

pub fn eth_recover_compressed() {
	use sp_core::{ecdsa, keccak_256, Pair};
	use sp_io::crypto::secp256k1_ecdsa_recover_compressed;

	let pair = ecdsa::Pair::from_string(&format!("//{}", 1), None).unwrap();
	let hash = keccak_256(b"Hello");
	let signature = pair.sign_prehashed(&hash);

	if let Ok(recovered_raw) = secp256k1_ecdsa_recover_compressed(&signature.0, &hash) {
		let recovered = ecdsa::Public::from_raw(recovered_raw);
		// Assert that we recovered the correct PK.
		assert_eq!(pair.public(), recovered);
	} else {
		panic!("recovery failed ...!");
	}
}
