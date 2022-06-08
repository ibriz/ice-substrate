#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
pub mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// All the types, traits defination and alises are inside this
pub mod types;

/// All independent utilities function are inside here
pub mod utils;

// Weight Information related to this palet
pub mod weights;

pub mod merkle;

#[cfg(not(feature = "no-vesting"))]
mod vested_transfer;

#[cfg(feature = "no-vesting")]
mod non_vested_transfer;

pub const MERKLE_ROOT: [u8; 32] =
	hex_literal::hex!("4c59b428da385567a6d42ee1881ecbe43cf30bf8c4499887b7c6f689d23d4672");

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use super::{types, utils, weights};
	use hex_literal::hex;
	use sp_runtime::traits::Convert;

	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
	use sp_std::prelude::*;

	use crate::merkle;
	use crate::types::MerkelProofValidator;
	use frame_support::storage::bounded_vec::BoundedVec;
	use frame_support::traits::{Currency, LockableCurrency, ReservableCurrency};
	use sp_runtime::traits::Verify;
	use weights::WeightInfo;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_vesting::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<types::AccountIdOf<Self>>
			+ ReservableCurrency<types::AccountIdOf<Self>>
			+ LockableCurrency<types::AccountIdOf<Self>>
			+ IsType<<Self as pallet_vesting::Config>::Currency>;

		/// Weight information for extrinsics in this pallet.
		type AirdropWeightInfo: WeightInfo;

		/// Type that allows back and forth conversion
		/// Airdrop Balance <==> Vesting Balance
		type BalanceTypeConversion: Convert<types::ServerBalance, types::BalanceOf<Self>>
			+ Convert<types::BalanceOf<Self>, types::ServerBalance>
			+ Convert<types::VestingBalanceOf<Self>, types::BalanceOf<Self>>
			+ Convert<types::BalanceOf<Self>, types::VestingBalanceOf<Self>>;

		type MerkelProofValidator: types::MerkelProofValidator<Self>;

		type MaxProofSize: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emit when an claim request was successful and fund have been transferred
		ClaimSuccess(types::IconAddress),

		/// Emit when an claim request was partially successful
		ClaimPartialSuccess(types::IconAddress),

		/// Value of ServerAccount sotrage have been changed
		/// Return old value and new one
		ServerAccountChanged {
			old_account: Option<types::AccountIdOf<T>>,
			new_account: types::AccountIdOf<T>,
		},

		/// AirdropState have been updated
		AirdropStateUpdated {
			old_state: types::AirdropState,
			new_state: types::AirdropState,
		},

		CreditorBalanceLow,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_airdrop_state)]
	pub(super) type AirdropChainState<T: Config> = StorageValue<_, types::AirdropState, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_icon_snapshot_map)]
	pub(super) type IceSnapshotMap<T: Config> =
		StorageMap<_, Twox64Concat, types::IconAddress, types::SnapshotInfo<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_airdrop_server_account)]
	pub(super) type ServerAccount<T: Config> = StorageValue<_, types::AccountIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_exchange_account)]
	pub type ExchangeAccountsMap<T: Config> =
		StorageMap<_, Twox64Concat, types::IconAddress, types::BalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn try_get_creditor_account)]
	// TODO:
	// Currently, putting this as ValueQuery means,
	// in case of no creditor account set, this storage will have
	// default address, 0x0000... in case of current sig type.
	//
	// Find a way to panic in such case at the first place.
	// Doing unwrap at runtime will probably be bad idea
	// and so will be to get default address
	// StorageValue<_, types::AccountIdOf<T>, ValueQuery, PanicOnNoCreditor>
	pub(super) type CreditorAccount<T: Config> =
		StorageValue<_, types::AccountIdOf<T>, OptionQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// This error will occur when signature validation failed.
		InvalidSignature,

		/// Error to return when unauthorised operation is attempted
		DeniedOperation,

		/// Not all data required are supplied with
		IncompleteData,

		/// Claim has already been made so can't be made again at this time
		ClaimAlreadyMade,

		/// Coversion between partially-compatible type failed
		FailedConversion,

		/// Creditor account do not have enough USABLE balance to
		/// undergo this transaction
		InsufficientCreditorBalance,

		/// Some operation while applying vesting failed
		CantApplyVesting,

		/// Currently no new claim request is being accepted
		NewClaimRequestBlocked,

		/// Currently processing of exchange request is blocked
		NewExchangeRequestBlocked,

		/// Given proof set was invalid to expected tree root
		InvalidMerkleProof,

		/// Provided proof size excced the maximum limit
		ProofTooLarge,

		InvalidIceAddress,
		InvalidIceSignature,
		FailedExtractingIceAddress,
		InvalidMessagePayload,
		ArithmeticError,

		/// Claim amount was not expected in this exchanged airdrop
		InvalidClaimAmount,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Dispatchable to be called by server with privileged account
		/// dispatch claim
		#[pallet::weight((
			T::AirdropWeightInfo::dispatch_user_claim(),
			DispatchClass::Normal,
			Pays::No
		))]
		pub fn dispatch_user_claim(
			origin: OriginFor<T>,
			icon_address: types::IconAddress,
			ice_address: types::IceAddress,
			message: types::RawPayload,
			icon_signature: types::IconSignature,
			ice_signature: types::IceSignature,
			total_amount: types::BalanceOf<T>,
			defi_user: bool,
			proofs: types::MerkleProofs<T>,
		) -> DispatchResultWithPostInfo {
			// Make sure only root or server account call call this
			Self::ensure_root_or_server(origin).map_err(|_| Error::<T>::DeniedOperation)?;

			// Make sure node is accepting new claimrequest
			Self::ensure_user_claim_switch()?;

			// Verify the integrity of message
			Self::validate_message_payload(&message, &ice_address)?;

			// We expect a valid proof of this exchange call
			Self::validate_merkle_proof(&icon_address, total_amount, defi_user, proofs)?;

			// Validate icon signature
			Self::validate_icon_address(&icon_address, &icon_signature, &message)?;

			// Validate ice signature
			Self::validate_ice_signature(&ice_signature, &icon_signature, &ice_address)?;

			// Now this address pair is verified,
			// we can insert it to the map if this pair is new
			let mut snapshot =
				Self::insert_or_get_snapshot(&icon_address, &ice_address, defi_user, total_amount);

			// Make sure this user is eligible for claim.
			Self::ensure_claimable(&snapshot)?;

			// We also make sure creditor have enough fund to complete this airdrop
			Self::validate_creditor_fund(total_amount)?;

			// Do the actual transfer if eligible
			Self::do_transfer(&mut snapshot, &icon_address, total_amount, defi_user)?;

			Self::deposit_event(Event::ClaimSuccess(icon_address));
			Ok(Pays::No.into())
		}

		#[pallet::weight((
			T::AirdropWeightInfo::dispatch_exchange_claim(),
			DispatchClass::Normal,
			Pays::No
		))]
		pub fn dispatch_exchange_claim(
			origin: OriginFor<T>,
			icon_address: types::IconAddress,
			ice_address: types::IceAddress,
			total_amount: types::BalanceOf<T>,
			defi_user: bool,
			proofs: types::MerkleProofs<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin).map_err(|_| Error::<T>::DeniedOperation)?;
			Self::ensure_exchange_claim_switch()?;

			let amount = Self::validate_whitelisted(&icon_address)?;
			ensure!(total_amount == amount, Error::<T>::InvalidClaimAmount);

			Self::validate_merkle_proof(&icon_address, total_amount, defi_user, proofs)?;
			Self::validate_creditor_fund(total_amount)?;

			let mut snapshot =
				Self::insert_or_get_snapshot(&icon_address, &ice_address, defi_user, total_amount);

			Self::ensure_claimable(&snapshot)?;
			Self::do_transfer(&mut snapshot, &icon_address, total_amount, defi_user)?;

			Self::deposit_event(Event::ClaimSuccess(icon_address));
			Ok(Pays::No.into())
		}

		#[pallet::weight(<T as Config>::AirdropWeightInfo::set_airdrop_server_account())]
		pub fn set_airdrop_server_account(
			origin: OriginFor<T>,
			new_account: types::AccountIdOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin).map_err(|_| Error::<T>::DeniedOperation)?;

			let old_account = Self::get_airdrop_server_account();
			<ServerAccount<T>>::set(Some(new_account.clone()));

			log::info!(
				"[Airdrop pallet] {} {:?}",
				"Value for ServerAccount was changed in onchain storage. (Old, New): ",
				(&old_account, &new_account)
			);

			Self::deposit_event(Event::ServerAccountChanged {
				old_account,
				new_account,
			});

			Ok(Pays::No.into())
		}

		#[pallet::weight(10_000)]
		pub fn update_airdrop_state(
			origin: OriginFor<T>,
			new_state: types::AirdropState,
		) -> DispatchResultWithPostInfo {
			// Only root can call this
			ensure_root(origin).map_err(|_| Error::<T>::DeniedOperation)?;

			let old_state = Self::get_airdrop_state();
			<AirdropChainState<T>>::set(new_state.clone());

			log::info!(
				"[Airdrop pallet] AirdropState updated. (Old, New): {:?} in block number {:?}",
				(&old_state, &new_state),
				Self::get_current_block_number(),
			);

			Self::deposit_event(Event::AirdropStateUpdated {
				old_state,
				new_state,
			});

			Ok(Pays::No.into())
		}
	}

	// implement all the helper function that are called from pallet dispatchable
	impl<T: Config> Pallet<T> {
		/// Check weather node is set to block incoming claim request
		/// Return error in that case else return Ok
		pub fn ensure_user_claim_switch() -> DispatchResult {
			let is_disabled = Self::get_airdrop_state().block_claim_request;

			if is_disabled {
				Err(Error::<T>::NewClaimRequestBlocked.into())
			} else {
				Ok(())
			}
		}

		pub fn get_creditor_account() -> types::AccountIdOf<T> {
			Self::try_get_creditor_account().expect("Creditor account not set")
		}

		/// Check weather node is set to block incoming exchange request
		/// Return error in that case else return Ok
		pub fn ensure_exchange_claim_switch() -> DispatchResult {
			let is_disabled = Self::get_airdrop_state().block_exchange_request;

			if is_disabled {
				Err(Error::<T>::NewExchangeRequestBlocked.into())
			} else {
				Ok(())
			}
		}

		/// Helper function to create similar interface like `ensure_root`
		/// but which instead check for server key
		pub fn ensure_root_or_server(origin: OriginFor<T>) -> DispatchResult {
			let is_root = ensure_root(origin.clone()).is_ok();
			let is_offchain = {
				let signed = ensure_signed(origin);
				signed.is_ok() && signed.ok() == Self::get_airdrop_server_account()
			};

			ensure!(is_root || is_offchain, DispatchError::BadOrigin);
			Ok(())
		}

		/// Return block height of Node from which this was called
		pub fn get_current_block_number() -> types::BlockNumberOf<T> {
			<frame_system::Pallet<T>>::block_number()
		}

		// Insert this address pair if it is new
		pub fn insert_or_get_snapshot(
			icon_address: &types::IconAddress,
			ice_address: &types::IceAddress,
			defi_user: bool,
			amount: types::BalanceOf<T>,
		) -> types::SnapshotInfo<T> {
			let old_snapshot = Self::get_icon_snapshot_map(icon_address);

			match old_snapshot {
				// As this pair is already on map,
				// we can just return it
				Some(old_snapshot) => old_snapshot,

				// This pair is new to the crew, add it
				None => {
					let mut new_snapshot =
						types::SnapshotInfo::<T>::default().ice_address(*ice_address);
					new_snapshot.amount = amount;
					new_snapshot.defi_user = defi_user;

					<IceSnapshotMap<T>>::insert(&icon_address, &new_snapshot);

					new_snapshot
				}
			}
		}

		pub fn ensure_claimable(snapshot: &types::SnapshotInfo<T>) -> DispatchResult {
			#[cfg(not(feature = "no-vesting"))]
			let already_claimed = snapshot.done_instant && snapshot.done_vesting;

			#[cfg(feature = "no-vesting")]
			let already_claimed = snapshot.done_instant;

			if already_claimed {
				Err(Error::<T>::ClaimAlreadyMade.into())
			} else {
				Ok(())
			}
		}

		pub fn validate_creditor_fund(required_amount: types::BalanceOf<T>) -> DispatchResult {
			let creditor_balance =
				<T as Config>::Currency::free_balance(&Self::get_creditor_account());
			let exestensial_deposit = <T as Config>::Currency::minimum_balance();

			if creditor_balance > required_amount + exestensial_deposit {
				Ok(())
			} else {
				Self::deposit_event(Event::<T>::CreditorBalanceLow);
				Err(Error::<T>::InsufficientCreditorBalance.into())
			}
		}

		pub fn validate_whitelisted(
			icon_address: &types::IconAddress,
		) -> Result<types::BalanceOf<T>, Error<T>> {
			Self::get_exchange_account(icon_address).ok_or(Error::<T>::DeniedOperation)
		}

		pub fn validate_icon_address(
			icon_address: &types::IconAddress,
			signature: &types::IconSignature,
			payload: &[u8],
		) -> Result<(), Error<T>> {
			let recovered_key = utils::recover_address(signature, payload)?;
			ensure!(
				recovered_key == icon_address.as_slice(),
				Error::<T>::InvalidSignature
			);
			Ok(())
		}

		pub fn validate_ice_signature(
			signature_raw: &[u8; 64],
			msg: &[u8],
			ice_bytes: &types::IceAddress,
		) -> Result<bool, Error<T>> {
			let wrapped_msg = utils::wrap_bytes(msg);

			let is_valid = Self::check_signature(signature_raw, &wrapped_msg, ice_bytes);
			if is_valid {
				Ok(true)
			} else {
				Err(Error::<T>::InvalidIceSignature)
			}
		}

		pub fn validate_message_payload(
			payload: &[u8],
			ice_address: &[u8; 32],
		) -> Result<(), Error<T>> {
			let extracted_address = utils::extract_ice_address(payload, ice_address)
				.map_err(|_e| Error::<T>::FailedExtractingIceAddress)?;
			ensure!(
				extracted_address == ice_address,
				Error::<T>::InvalidMessagePayload
			);
			Ok(())
		}

		pub fn check_signature(
			signature_raw: &[u8; 64],
			msg: &[u8],
			account_bytes: &[u8; 32],
		) -> bool {
			let signature = sp_core::sr25519::Signature::from_raw(*signature_raw);
			let public = sp_core::sr25519::Public::from_raw(*account_bytes);
			signature.verify(msg, &public)
		}

		pub fn get_bounded_proofs(
			input: Vec<types::MerkleHash>,
		) -> Result<BoundedVec<types::MerkleHash, T::MaxProofSize>, Error<T>> {
			let bounded_vec = BoundedVec::<types::MerkleHash, T::MaxProofSize>::try_from(input)
				.map_err(|()| Error::<T>::ProofTooLarge)?;
			Ok(bounded_vec)
		}

		pub fn validate_merkle_proof(
			icon_address: &types::IconAddress,
			amount: types::BalanceOf<T>,
			defi_user: bool,
			proof_hashes: types::MerkleProofs<T>,
		) -> Result<bool, Error<T>> {
			let amount = types::from_balance::<T>(amount);
			let leaf_hash = merkle::hash_leaf(icon_address, amount, defi_user);
			let is_valid_proof = <T as Config>::MerkelProofValidator::validate(
				leaf_hash,
				crate::MERKLE_ROOT,
				proof_hashes,
			);
			if !is_valid_proof {
				return Err(Error::<T>::InvalidMerkleProof);
			}

			Ok(true)
		}

		pub fn to_account_id(ice_bytes: [u8; 32]) -> Result<types::AccountIdOf<T>, Error<T>> {
			<T as frame_system::Config>::AccountId::decode(&mut &ice_bytes[..])
				.map_err(|_e| Error::<T>::InvalidIceAddress)
		}

		pub fn do_transfer(
			snapshot: &mut types::SnapshotInfo<T>,
			icon_address: &types::IconAddress,
			total_amount: types::BalanceOf<T>,
			defi_user: bool,
		) -> Result<(), DispatchError> {
			use types::DoTransfer;

			#[cfg(not(feature = "no-vesting"))]
			type TransferType = super::vested_transfer::DoVestdTransfer;

			#[cfg(feature = "no-vesting")]
			type TransferType = super::non_vested_transfer::AllInstantTransfer;

			let transfer_result =
				TransferType::do_transfer(snapshot, icon_address, total_amount, defi_user);

			// No matter the result we will write the updated_snapshot
			<IceSnapshotMap<T>>::insert(icon_address, snapshot);

			// Now snapshot have been written, return result
			transfer_result
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<T: Config> Pallet<T> {
		pub fn init_balance(account: &types::AccountIdOf<T>, free: types::ServerBalance) {
			let amount = <T::BalanceTypeConversion as Convert<_, _>>::convert(free);
			<T as Config>::Currency::make_free_balance_be(account, amount);
		}

		pub fn set_creditor_account(new_account: sr25519::Public) {
			let mut account_bytes = new_account.0.clone();
			let account = T::AccountId::decode(&mut &account_bytes[..]).unwrap_or_default();

			<CreditorAccount<T>>::set(Some(account.clone()));
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub exchange_accounts: Vec<(types::IconAddress, types::BalanceOf<T>)>,
		pub creditor_account: types::AccountIdOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			let creditor_account_hex =
				hex!["d893ef775b5689473b2e9fa32c1f15c72a7c4c86f05f03ee32b8aca6ce61b92c"];
			let creditor_account =
				types::AccountIdOf::<T>::decode(&mut &creditor_account_hex[..]).unwrap();

			let exchange_accounts = vec![];

			Self {
				exchange_accounts,
				creditor_account,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (address, balance) in &self.exchange_accounts {
				<ExchangeAccountsMap<T>>::insert(address, balance);
			}

			CreditorAccount::<T>::put(&self.creditor_account);
		}
	}
}
