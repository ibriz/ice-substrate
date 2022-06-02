use crate as airdrop;
use airdrop::pallet::Config;
use airdrop::pallet::Error;
use codec::MaxEncodedLen;
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

pub type ServerBalance = u128;

pub fn to_balance<T: Config>(amount: ServerBalance) -> BalanceOf<T> {
	<T::BalanceTypeConversion as Convert<ServerBalance, BalanceOf<T>>>::convert(amount)
}

pub fn from_balance<T: Config>(amount: BalanceOf<T>) -> ServerBalance {
	<T::BalanceTypeConversion as Convert<BalanceOf<T>, ServerBalance>>::convert(amount)
}

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
#[derive(Eq, PartialEq, Debug)]
pub enum SignatureValidationError {
	InvalidIconAddress,
	InvalidIconSignature,
	InvalidIceAddress,
	Sha3Execution,
}

#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen, Debug)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
#[derive(Eq, PartialEq)]
pub struct SnapshotInfo<T: Config> {
	/// Icon address of this snapshot
	pub ice_address: IceAddress,

	/// Total airdroppable-amount this icon_address hold
	pub amount: BalanceOf<T>,

	/// Indicator wather this icon_address holder is defi-user
	pub defi_user: bool,

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
	pub fn ice_address(mut self, val: IceAddress) -> Self {
		self.ice_address = val;
		self
	}
}

/// implement default values for snapshotInfo
impl<T: Config> Default for SnapshotInfo<T> {
	fn default() -> Self {
		Self {
			ice_address: [0u8; 32],
			amount: 0_u32.into(),
			defi_user: false,
			done_instant: false,
			done_vesting: false,
			vesting_block_number: None,
			initial_transfer: BalanceOf::<T>::from(0u32),
		}
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

pub fn balance_to_u32<T: Config>(input: BalanceOf<T>) -> u32 {
	TryInto::<u32>::try_into(input).ok().unwrap()
}

pub fn block_number_to_u32<T: Config>(input: BlockNumberOf<T>) -> u32 {
	TryInto::<u32>::try_into(input).ok().unwrap()
}
/// Chain state
#[derive(Deserialize, Encode, Decode, Clone, Eq, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct AirdropState {
	// Only receive claim request when this flag is true
	pub block_claim_request: bool,

	// Only receive exchange request when this flag is true
	pub block_exchange_request: bool,
}

impl Default for AirdropState {
	fn default() -> Self {
		AirdropState {
			block_claim_request: false,
			block_exchange_request: false,
		}
	}
}

pub trait MerkelProofValidator<T: Config> {
	fn validate(leaf_hash: MerkleHash, root_hash: MerkleHash, proofs: MerkleProofs<T>) -> bool;
}

/// Trait to commit behaviour of do_transfer function
/// this trait now can me implmeneted according to
/// the node behaviour eg: vesting manner and direct manner
pub trait DoTransfer {
	fn do_transfer<T: Config>(
		snapshot: &mut SnapshotInfo<T>,
		icon_address: &IconAddress,
		total_amount: BalanceOf<T>,
		defi_user: bool,
	) -> Result<(), DispatchError>;
}
