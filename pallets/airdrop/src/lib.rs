#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
pub mod mock;

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

use hex_literal::hex;

/// An identifier for a type of cryptographic key.
/// For this pallet, account associated with this key must be same as
/// Key stored in pallet_sudo. So that the calls made from offchain worker
/// won't get discarded because of Denied Operation
pub const KEY_TYPE_ID: sp_runtime::KeyTypeId = sp_runtime::KeyTypeId(*b"_air");

/// Gap between on when to run offchain owrker between
/// This is the number of ocw-run block to skip after running offchain worker
/// Eg: if block is ran on block_number=3 then
/// run offchain worker in 3+ENABLE_IN_EVERY block
pub const OFFCHAIN_WORKER_BLOCK_GAP: u32 = 3;

// Maximum number of time to retry a failed processing of claim entry
// There is NO point of seeting this to high value
pub const DEFAULT_RETRY_COUNT: u8 = 2;

pub const MERKLE_ROOT: [u8; 32] =
	hex_literal::hex!("4c59b428da385567a6d42ee1881ecbe43cf30bf8c4499887b7c6f689d23d4672");

#[frame_support::pallet]
pub mod pallet {
	use super::{types, utils, weights};
	use sp_runtime::traits::{CheckedAdd, Convert};

	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
	use sp_std::prelude::*;

	use crate::merkle;
	use crate::types::MerkelProofValidator;
	use frame_support::storage::bounded_vec::BoundedVec;
	use frame_support::traits::{
		Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency,
	};
	use frame_system::offchain::CreateSignedTransaction;
	use sp_runtime::traits::{IdentifyAccount, Member, Verify};
	use types::IconVerifiable;
	use weights::WeightInfo;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_vesting::Config {
		/// AccountIf type that is same as frame_system's accountId also
		/// extended to be verifable against icon data
		type VerifiableAccountId: IconVerifiable + IsType<<Self as frame_system::Config>::AccountId>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<types::AccountIdOf<Self>>
			+ ReservableCurrency<types::AccountIdOf<Self>>
			+ LockableCurrency<types::AccountIdOf<Self>>
			+ IsType<<Self as pallet_vesting::Config>::Currency>;

		#[deprecated(note = "Do tight coupling or expanded loose coupling of vesting_pallet")]
		type VestingModule: pallet_vesting::Config + IsType<Self>;

		/// The overarching dispatch call type.
		// type Call: From<Call<Self>>;

		/// The identifier type for an offchain worker.
		// type AuthorityId: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>;

		/// Weight information for extrinsics in this pallet.
		type AirdropWeightInfo: WeightInfo;

		/// Type that allows back and forth conversion
		/// Server Balance type <==> Airdrop Balance <==> Vesting Balance
		type BalanceTypeConversion: Convert<types::ServerBalance, types::BalanceOf<Self>>
			+ Convert<types::ServerBalance, types::VestingBalanceOf<Self>>
			+ Convert<types::VestingBalanceOf<Self>, types::BalanceOf<Self>>;

		/// Endpoint on where to send request url
		#[pallet::constant]
		type FetchIconEndpoint: Get<&'static str>;

		/// Id of account from which to send fund to claimers
		/// This account should be credited enough to supply fund for all claim requests
		#[pallet::constant]
		type Creditor: Get<frame_support::PalletId>;

		type MerkelProofValidator: types::MerkelProofValidator<Self>;

		type MaxProofSize: Get<u32>;

		type Public: IdentifyAccount<AccountId = Self::VerifiableAccountId> + Clone;
		type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
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

		DonatedToCreditor(types::AccountIdOf<T>, types::BalanceOf<T>),

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
		StorageMap<_, Twox64Concat, types::IconAddress, u64, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn creditor_account)]
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

		FailedConversion,

		InsufficientCreditorBalance,

		/// Some operation while applying vesting failed
		CantApplyVesting,

		/// Currently no new claim request is being accepted
		NewClaimRequestBlocked,

		/// Currently processing of exchange request is blocked
		NewExchangeRequestBlocked,

		ArithmeticError,

		FailedMappingAccount,

		InvalidMerkleProof,

		ProofTooLarge,
		InvalidIceAddress,
		InvalidIceSignature,
		FailedExtractingIceAddress,
		InvalidMessagePayload,
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
			message: Vec<u8>,
			icon_signature: types::IconSignature,
			ice_signature: types::IceSignature,
			total_amount: types::ServerBalance,
			defi_user: bool,
			proofs: types::MerkleProofs<T>,
		) -> DispatchResultWithPostInfo {
			// Make sure its callable by sudo or offchain
			Self::ensure_root_or_server(origin.clone()).map_err(|_| Error::<T>::DeniedOperation)?;

			Self::ensure_request_acceptance()?;

			Self::validate_message_payload(&message, &ice_address)?;

			Self::validate_merkle_proof(&icon_address, total_amount, defi_user, proofs)?;

			Self::validate_icon_address(&icon_address, &icon_signature, &message)?;

			Self::validate_ice_signature(&ice_signature, &icon_signature, &ice_address)?;

			Self::validate_creditor_fund(total_amount)?;
			//  write starts here so payload should be validated before this.
			let mut snapshot =
				Self::validate_unclaimed(&icon_address, &ice_address, total_amount, defi_user)?;
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
			total_amount: types::ServerBalance,
			defi_user: bool,
			proofs: types::MerkleProofs<T>,
		) -> DispatchResultWithPostInfo {
			// Make sure its callable by sudo or offchain
			ensure_root(origin.clone()).map_err(|_| Error::<T>::DeniedOperation)?;
			Self::ensure_exchange_acceptance()?;

			let amount = Self::validate_whitelisted(&icon_address)?;
			ensure!(total_amount == amount, Error::<T>::InvalidClaimAmount);

			Self::validate_merkle_proof(&icon_address, total_amount, defi_user, proofs)?;
			Self::validate_creditor_fund(total_amount)?;

			// check if claim has already been processed
			let mut snapshot =
				Self::validate_unclaimed(&icon_address, &ice_address, total_amount, defi_user)?;
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

		/// Public function to deposit some fund for our creditor
		/// @parameter:
		/// - origin: Signed Origin from which to credit
		/// - amount: Amount to donate
		/// - allow_death: when transferring amount,
		/// 		if origin's balance drop below minimum balance
		/// 		then weather to transfer (resulting origin account to vanish)
		/// 		or cancel the donation
		/// This function can be used as a mean to credit our creditor if being donated from
		/// any node operator owned account

		#[pallet::weight(<T as Config>::AirdropWeightInfo::donate_to_creditor(types::balance_to_u32::<T>(amount.clone())))]
		pub fn donate_to_creditor(
			origin: OriginFor<T>,
			amount: types::BalanceOf<T>,
			allow_death: bool,
		) -> DispatchResult {
			let sponser = ensure_signed(origin)?;
			let amount = types::BalanceOf::<T>::from(amount);

			let creditor_account = Self::get_creditor_account();
			let existance_req = if allow_death {
				ExistenceRequirement::AllowDeath
			} else {
				ExistenceRequirement::KeepAlive
			};

			<T as Config>::Currency::transfer(&sponser, &creditor_account, amount, existance_req)?;

			Self::deposit_event(Event::<T>::DonatedToCreditor(sponser, amount));

			Ok(())
		}
	}

	// implement all the helper function that are called from pallet dispatchable
	impl<T: Config> Pallet<T> {
		pub fn get_creditor_account() -> types::AccountIdOf<T> {
			Self::creditor_account().expect("Creditor Account Not Set")
		}

		pub fn ensure_request_acceptance() -> DispatchResult {
			let is_disabled = Self::get_airdrop_state().block_claim_request;

			if is_disabled {
				Err(Error::<T>::NewClaimRequestBlocked.into())
			} else {
				Ok(())
			}
		}

		pub fn ensure_exchange_acceptance() -> DispatchResult {
			let is_disabled = Self::get_airdrop_state().block_exchange_request;

			if is_disabled {
				Err(Error::<T>::NewExchangeRequestBlocked.into())
			} else {
				Ok(())
			}
		}

		/// Helper function to create similar interface like `ensure_root`
		/// but which instead check for sudo key
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

		#[cfg(not(feature = "no-vesting"))]
		pub fn validate_unclaimed(
			icon_address: &types::IconAddress,
			ice_address: &types::IceAddress,
			amount: types::ServerBalance,
			defi_user: bool,
		) -> Result<types::SnapshotInfo<T>, Error<T>> {
			let snapshot = Self::get_icon_snapshot_map(icon_address);
			if let Some(saved) = snapshot {
				if saved.done_vesting && saved.done_instant {
					return Err(Error::<T>::ClaimAlreadyMade.into());
				}
				return Ok(saved);
			}

			let mut new_snapshot =
				types::SnapshotInfo::<T>::default().ice_address(ice_address.clone());

			new_snapshot.defi_user = defi_user;
			new_snapshot.amount = types::to_balance::<T>(amount);

			<IceSnapshotMap<T>>::insert(&icon_address, &new_snapshot);

			Ok(new_snapshot)
		}

		#[cfg(feature = "no-vesting")]
		pub fn validate_unclaimed(
			icon_address: &types::IconAddress,
			ice_address: &types::IceAddress,
			amount: types::ServerBalance,
			defi_user: bool,
		) -> Result<types::SnapshotInfo<T>, Error<T>> {
			let snapshot = Self::get_icon_snapshot_map(icon_address);
			if let Some(saved) = snapshot {
				if saved.done_instant {
					return Err(Error::<T>::ClaimAlreadyMade.into());
				}
				return Ok(saved);
			}

			let mut new_snapshot =
				types::SnapshotInfo::<T>::default().ice_address(ice_address.clone());

			new_snapshot.defi_user = defi_user;
			new_snapshot.amount = types::to_balance::<T>(amount);

			<IceSnapshotMap<T>>::insert(&icon_address, &new_snapshot);

			Ok(new_snapshot)
		}

		pub fn validate_creditor_fund(amount: types::ServerBalance) -> DispatchResult {
			let creditor_balance =
				<T as Config>::Currency::free_balance(&Self::get_creditor_account());
			let required_amount = types::to_balance::<T>(amount);
			let exestensial_deposit = <T as Config>::Currency::minimum_balance();

			if creditor_balance > required_amount + exestensial_deposit {
				Ok(())
			} else {
				Self::deposit_event(Event::<T>::CreditorBalanceLow);
				Err(Error::<T>::InsufficientCreditorBalance.into())
			}
		}

		pub fn validate_whitelisted(icon_address: &types::IconAddress) -> Result<u64, Error<T>> {
			Self::get_exchange_account(icon_address).ok_or_else(|| Error::<T>::DeniedOperation)
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
				Err(Error::<T>::InvalidIceSignature.into())
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
			let signature = sp_core::sr25519::Signature::from_raw(signature_raw.clone());
			let public = sp_core::sr25519::Public::from_raw(account_bytes.clone());
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
			amount: types::ServerBalance,
			defi_user: bool,
			proof_hashes: types::MerkleProofs<T>,
		) -> Result<bool, Error<T>> {
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

		/// Split total amount to chunk of 3 amount
		/// These are the amounts that are to be vested in next
		/// 3 lot.
		pub fn get_splitted_amounts(
			total_amount: types::ServerBalance,
			is_defi_user: bool,
		) -> Result<(types::BalanceOf<T>, types::VestingBalanceOf<T>), DispatchError> {
			const DEFI_INSTANT_PER: u32 = 40_u32;
			const NORMAL_INSTANT_PER: u32 = 30_u32;

			let percentage = if is_defi_user {
				DEFI_INSTANT_PER
			} else {
				NORMAL_INSTANT_PER
			};

			let instant_amount = total_amount
				.checked_mul(percentage.into())
				.ok_or(sp_runtime::ArithmeticError::Overflow)?
				.checked_div(100_u32.into())
				.ok_or(sp_runtime::ArithmeticError::Underflow)?;

			let vesting_amount = total_amount
				.checked_sub(instant_amount)
				.ok_or(sp_runtime::ArithmeticError::Underflow)?;

			Ok((
				<T::BalanceTypeConversion as Convert<_, _>>::convert(instant_amount),
				<T::BalanceTypeConversion as Convert<_, _>>::convert(vesting_amount),
			))
		}

		#[cfg(feature = "no-vesting")]
		pub fn do_transfer(
			snapshot: &mut types::SnapshotInfo<T>,
			icon_address: &types::IconAddress,
			total_amount: types::ServerBalance,
			defi_user: bool,
		) -> Result<(), DispatchError> {
			let total_balance = <T::BalanceTypeConversion as Convert<
				types::ServerBalance,
				types::BalanceOf<T>,
			>>::convert(total_amount);
			let creditor = Self::get_creditor_account();
			let claimer = Self::to_account_id(snapshot.ice_address.clone())?;
			if !snapshot.done_instant {
				<T as Config>::Currency::transfer(
					&creditor,
					&claimer,
					total_balance,
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|err| {
					log::error!(
						"[Airdrop pallet] Cannot instant transfer to {:?}. Reason: {:?}",
						claimer,
						err
					);
					err
				})?;

				// Everything went ok. Update flag
				snapshot.done_instant = true;
				snapshot.initial_transfer = total_balance;
				<IceSnapshotMap<T>>::insert(&icon_address, snapshot.clone());
			} else {
				log::trace!(
					"[Airdrop pallet] Doing instant transfer for {:?} skipped in {:?}",
					claimer,
					Self::get_current_block_number()
				);
			}
			Ok(())
		}

		#[cfg(not(feature = "no-vesting"))]
		pub fn do_transfer(
			snapshot: &mut types::SnapshotInfo<T>,
			icon_address: &types::IconAddress,
			total_amount: types::ServerBalance,
			defi_user: bool,
		) -> Result<(), DispatchError> {
			// TODO: put more relaible value
			const BLOCKS_IN_YEAR: u32 = 5_256_000u32;
			// Block number after which enable to do vesting
			const VESTING_APPLICABLE_FROM: u32 = 1u32;
			let claimer = snapshot.ice_address.clone();
			let creditor = Self::get_creditor_account();

			let (mut instant_amount, vesting_amount) =
				Self::get_splitted_amounts(total_amount, defi_user)?;

			let (transfer_shcedule, remainding_amount) = utils::new_vesting_with_deadline::<
				T,
				VESTING_APPLICABLE_FROM,
			>(vesting_amount, BLOCKS_IN_YEAR.into());

			// Amount to be transferred is:
			// x% of totoal amount
			// + remainding amount which was not perfectly divisible
			instant_amount = {
				let remainding_amount = <T::BalanceTypeConversion as Convert<
					types::VestingBalanceOf<T>,
					types::BalanceOf<T>,
				>>::convert(remainding_amount);

				instant_amount
					.checked_add(&remainding_amount)
					.ok_or(sp_runtime::ArithmeticError::Overflow)?
			};

			let creditor_origin = <T as frame_system::Config>::Origin::from(
				frame_system::RawOrigin::Signed(creditor.clone()),
			);
			let claimer_account = Self::to_account_id(claimer)?;
			let claimer_origin =
				<T::Lookup as sp_runtime::traits::StaticLookup>::unlookup(claimer_account.clone());

			match transfer_shcedule {
				// Apply vesting
				Some(schedule) if !snapshot.done_vesting => {
					let vest_res = pallet_vesting::Pallet::<T>::vested_transfer(
						creditor_origin.clone(),
						claimer_origin.clone(),
						schedule,
					);

					match vest_res {
						// Everything went ok. update flag
						Ok(()) => {
							snapshot.done_vesting = true;
							snapshot.vesting_block_number = Some(Self::get_current_block_number());
							<IceSnapshotMap<T>>::insert(&icon_address, snapshot.clone());
							log::info!("[Airdrop pallet] Vesting applied for {:?}", claimer);
						}
						// log error
						Err(err) => {
							log::info!(
								"[Airdrop pallet] Applying vesting for {:?} failed with error: {:?}",
								claimer,
								err
							);
						}
					}
				}

				// Vesting was already done as snapshot.done_vesting is true
				Some(_) => {
					log::trace!(
						"[Airdrop pallet] Doing instant transfer for {:?} skipped in {:?}",
						claimer,
						Self::get_current_block_number()
					);
				}

				// No schedule was created
				None => {
					// If vesting is not applicable once then with same total_amount
					// it will not be applicable ever. So mark it as done.
					snapshot.done_vesting = true;
					snapshot.vesting_block_number = Some(Self::get_current_block_number());
					<IceSnapshotMap<T>>::insert(&icon_address, snapshot.clone());

					log::trace!(
						"[Airdrop pallet] Primary vesting not applicable for {:?}",
						claimer_origin,
					);
				}
			}

			// if not done previously
			// Transfer the amount user is expected to receiver instantly
			if !snapshot.done_instant {
				<T as Config>::Currency::transfer(
					&creditor,
					&claimer_account,
					instant_amount,
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|err| {
					// First reason to fail this transfer is due to low balance in creditor. Althogh
					// there are other reasons why it might fail:
					// - instant_amount is too low to transfer. This can be prevented by making sure
					// that we have a certain lower bound for airdropping so that
					// this instant_amount will never be too low to transfer
					// - this is the very first operation on `ice-address` and instant_transfer is less than
					// exestinsial deposit for ice_address to exist.
					log::error!(
						"[Airdrop pallet] Cannot instant transfer to {:?}. Reason: {:?}",
						claimer,
						err
					);
					err
				})?;

				// Everything went ok. Update flag
				snapshot.done_instant = true;
				snapshot.initial_transfer = instant_amount;
				<IceSnapshotMap<T>>::insert(&icon_address, snapshot.clone());
			} else {
				log::trace!(
					"[Airdrop pallet] Doing instant transfer for {:?} skipped in {:?}",
					claimer,
					Self::get_current_block_number()
				);
			}

			Ok(())
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
		pub exchange_accounts: Vec<(types::IconAddress, u64)>,
		pub creditor_account: Option<types::AccountIdOf<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				exchange_accounts: Vec::new(),
				creditor_account: None,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for account in &self.exchange_accounts {
				<ExchangeAccountsMap<T>>::insert(account.0, account.1);
			}
			if let Some(ref key) = self.creditor_account {
				CreditorAccount::<T>::put(key);
			}
		}
	}
}
