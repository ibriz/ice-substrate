use crate as airdrop;
use airdrop::pallet::Config;
use airdrop::pallet::Error;
use core::convert::Into;
use frame_support::pallet_prelude::*;
use frame_support::traits::Currency;
use frame_system;
use scale_info::TypeInfo;
use serde::Deserialize;
use sp_core::H160;
use sp_runtime::traits::Convert;
use sp_runtime::ArithmeticError;
use sp_std::prelude::*;

use frame_support::storage::bounded_vec::BoundedVec;

/// AccountId of anything that implements frame_system::Config
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

///
pub type VestingBalanceOf<T> =
	<<T as pallet_vesting::Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

/// Type that represent the balance
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

pub type SignatureOf<T> = <T as Config>::Signature;

/// Balance type that will be returned from server
pub type ServerBalance = u64;

/// Type that represent IconAddress
pub type IconAddress = [u8; 20];

pub type IceAddress = [u8; 32];

pub type IceEvmAddress = H160;

/// Type that represent Icon signed message
pub type IconSignature = [u8; 65];

pub type IceSignature = [u8; 64];

///
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

pub type MerkleHash = [u8; 32];
// pub type MerkleProofs=Vec<MerkleHash>;
pub type MerkleProofs<T> = BoundedVec<MerkleHash, <T as Config>::MaxProofSize>;

///
pub type VestingInfoOf<T> = pallet_vesting::VestingInfo<VestingBalanceOf<T>, BlockNumberOf<T>>;

/// type that represnt the error that can occur while validation the signature
#[derive(Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
pub enum SignatureValidationError {
	InvalidIconAddress,
	InvalidIconSignature,
	InvalidIceAddress,
	Sha3Execution,
}

#[derive(Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
#[derive(Eq, PartialEq)]
pub struct SnapshotInfo<T: Config> {
	/// Icon address of this snapshot
	// TODO:
	// change this to [u8; _]
	pub ice_address: AccountIdOf<T>,

	/// Total airdroppable-amount this icon_address hold
	pub amount: BalanceOf<T>,

	/// Indicator wather this icon_address holder is defi-user
	pub defi_user: bool,

	/// TODO: add description of this filed
	pub vesting_percentage: u32,

	/// indicator wather the user have claimmed the balance
	/// which will be given through instant transfer
	pub done_instant: bool,

	/// Indicator weather vesting schedult have been applied
	/// to this user
	pub done_vesting: bool,

	// blocknumber that started vesting
	pub vesting_block_number: Option<BlockNumberOf<T>>,

	pub initial_transfer: BalanceOf<T>,
}

impl<T: Config> SnapshotInfo<T> {
	/// Helper function to set ice_address in builder-pattern way
	/// so that initilisation can be done in single line
	pub fn ice_address(mut self, val: AccountIdOf<T>) -> Self {
		self.ice_address = val;
		self
	}
}

/// implement default values for snapshotInfo
impl<T: Config> Default for SnapshotInfo<T> {
	fn default() -> Self {
		Self {
			ice_address: AccountIdOf::<T>::default(),
			amount: 0_u32.into(),
			defi_user: false,
			vesting_percentage: 0,
			done_instant: false,
			done_vesting: false,
			vesting_block_number: None,
			initial_transfer: BalanceOf::<T>::from(0u32),
		}
	}
}

/// Possible values of error that can occur when doing claim request from offchain worker
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
#[derive(PartialEq, Eq)]
pub enum ClaimError {
	/// When there is no icon address in mapping corresponding
	/// to the ice_address stored in queue
	NoIconAddress,

	/// Error while doing an http request
	/// Also might contains the actual error
	HttpError,
	/// Server returned an response in a format that couldn't be understood
	/// this is set when response neither could not be deserialize into
	/// valid server response or valid server error
	InvalidResponse,

	/// Error was occured when making extrinsic call
	CallingError(CallDispatchableError),
}

/// Structure expected to return from server when doing a request for details of icon_address
#[derive(Deserialize, Encode, Decode, Clone, Default, Eq, PartialEq, TypeInfo, Copy)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ServerResponse {
	// TODO: Add description of this field
	pub omm: ServerBalance,

	/// Amount to transfer in this claim
	#[serde(rename = "balanced")]
	pub amount: ServerBalance,

	// TODO: add description of this field
	pub stake: ServerBalance,

	/// Indicator weather this icon_address is defi_user or not
	pub defi_user: bool,
}

impl ServerResponse {
	pub fn get_total_balance<T: Config>(&self) -> Result<BalanceOf<T>, Error<T>> {
		let total = self.get_total().map_err(|e| Error::<T>::from(e))?;
		let balance =
			<T::BalanceTypeConversion as Convert<ServerBalance, BalanceOf<T>>>::convert(total);
		Ok(balance)
	}

	pub fn get_total(&self) -> Result<ServerBalance, ArithmeticError> {
		use sp_runtime::ArithmeticError::Overflow;
		let total = self
			.amount
			.checked_add(self.stake)
			.ok_or(Overflow)?
			.checked_add(self.omm)
			.ok_or(Overflow);
		total
	}
}

impl<T: Config> From<ArithmeticError> for Error<T> {
	fn from(_: ArithmeticError) -> Self {
		Error::<T>::ArithmeticError
	}
}

impl<T: Config> From<SignatureValidationError> for Error<T> {
	fn from(_: SignatureValidationError) -> Self {
		Error::<T>::InvalidSignature
	}
}

pub fn to_balance<T: Config>(amount: ServerBalance) -> BalanceOf<T> {
	<T::BalanceTypeConversion as Convert<ServerBalance, BalanceOf<T>>>::convert(amount)
}

/// Error while calling On-chain calls from offchain worker
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
#[derive(Eq, PartialEq)]
pub enum CallDispatchableError {
	/// No any account was found to send signed transaction from
	NoAccount,

	/// Error while dispatching the call
	CantDispatch,
}

/// Trait that marks something is verifable agains the given icon data
// This was originally created to be implemented against AccountId of airdrop-pallet
// as a way to ensure that the ice & icon address pair is authorised
pub trait IconVerifiable {
	fn verify_with_icon(
		&self,
		icon_wallet: &IconAddress,
		icon_signature: &IconSignature,
		message: &[u8],
	) -> Result<(), SignatureValidationError>;
}

pub fn balance_to_u32<T: Config>(input: BalanceOf<T>) -> u32 {
	TryInto::<u32>::try_into(input).ok().unwrap()
}

pub fn block_number_to_u32<T: Config>(input: BlockNumberOf<T>) -> u32 {
	TryInto::<u32>::try_into(input).ok().unwrap()
}
/// Chain state
#[derive(Deserialize, Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct AirdropState {
	// Only receive claim request when this flag is true
	pub block_claim_request: bool,

	// Only process already received claim request when this flag is true
	#[deprecated(note = "This pallet no longer user offchain so this flag is not needed")]
	pub avoid_claim_processing: bool,
}

impl Default for AirdropState {
	fn default() -> Self {
		AirdropState {
			block_claim_request: false,
			avoid_claim_processing: false,
		}
	}
}

pub trait MerkelProofValidator<T: Config> {
	fn validate(
		icon_address: &IconAddress,
		amount: u64,
		defi_user: bool,
		root_hash: MerkleHash,
		leaf_hash: MerkleHash,
		proofs: MerkleProofs<T>,
	) -> bool;

	// fn proof_root(leaf_hash: types::MerkleHash, proofs: types::MerkleProofs<T>) -> [u8; 32];
}
