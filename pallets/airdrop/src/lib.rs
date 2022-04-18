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

#[frame_support::pallet]
pub mod pallet {
	use super::{types, utils, weights};
	use sp_runtime::traits::{CheckedAdd, Convert, Saturating};

	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
	use sp_std::prelude::*;

	use frame_support::traits::{
		Currency, ExistenceRequirement, Hooks, LockableCurrency, ReservableCurrency,
	};
	use frame_system::offchain::CreateSignedTransaction;
	use types::IconVerifiable;
	use weights::WeightInfo;
	use sp_runtime::ArithmeticError;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + CreateSignedTransaction<Call<Self>> + pallet_vesting::Config
	{
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
		type AuthorityId: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>;

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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emit when a claim request have been removed from queue
		ClaimCancelled(types::IconAddress),

		/// Emit when claim request was done successfully
		ClaimRequestSucced {
			ice_address: types::AccountIdOf<T>,
			icon_address: types::IconAddress,
			registered_in: types::BlockNumberOf<T>,
		},

		/// Emit when an claim request was successful and fund have been transferred
		ClaimSuccess(types::IconAddress),

		/// An entry from queue was removed
		RemovedFromQueue(types::IconAddress),

		DonatedToCreditor(types::AccountIdOf<T>, types::BalanceOf<T>),

		RegisteredFailedClaim(types::AccountIdOf<T>, types::BlockNumberOf<T>, u8),

		/// Same entry is processed by offchian worker for too many times
		RetryExceed {
			ice_address: types::AccountIdOf<T>,
			icon_address: types::IconAddress,
			was_in: types::BlockNumberOf<T>,
		},

		/// Value of OffchainAccount sotrage have been changed
		/// Return old value and new one
		OffchainAccountChanged {
			old_account: Option<types::AccountIdOf<T>>,
			new_account: types::AccountIdOf<T>,
		},

		/// AirdropState have been updated
		AirdropStateUpdated {
			old_state: types::AirdropState,
			new_state: types::AirdropState,
		},

		/// Processed upto counter was updated to new value
		ProcessedCounterSet(types::BlockNumberOf<T>),

		CreditorBalanceLow,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_pending_claims)]
	pub(super) type PendingClaims<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::BlockNumber,
		Twox64Concat,
		types::IconAddress,
		u8,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_processed_upto_counter)]
	pub(super) type ProcessedUpto<T: Config> = StorageValue<_, types::BlockNumberOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_airdrop_state)]
	pub(super) type AirdropChainState<T: Config> = StorageValue<_, types::AirdropState, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_icon_snapshot_map)]
	pub(super) type IceSnapshotMap<T: Config> =
		StorageMap<_, Twox64Concat, types::IconAddress, types::SnapshotInfo<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_offchain_account)]
	pub(super) type OffchainAccount<T: Config> =
		StorageValue<_, types::AccountIdOf<T>, OptionQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// This error will occur when signature validation failed.
		InvalidSignature,

		/// Error to return when unauthorised operation is attempted
		DeniedOperation,

		/// Not all data required are supplied with
		IncompleteData,

		/// When expected data is not present in queue
		NotInQueue,

		/// Claim has already been made so can't be made again at this time
		ClaimAlreadyMade,

		/// This request have been already made.
		RequestAlreadyMade,

		/// When a same entry is being retried for too many times
		RetryExceed,

		FailedConversion,

		InsufficientCreditorBalance,
	

		/// Some operation while applying vesting failed
		CantApplyVesting,

		/// Creditor have too low balance
		CreditorOutOfFund,

		/// Currently no new claim request is being accepted
		NewClaimRequestBlocked,

		/// Currently processing of claim request is blocked
		ClaimProcessingBlocked,

		ArithmeticError,
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
			ice_address: types::AccountIdOf<T>,
			message: Vec<u8>,
			icon_signature: types::IconSignature,
			server_response: types::ServerResponse,
		) -> DispatchResultWithPostInfo {
			// Make sure its callable by sudo or offchain
			Self::ensure_root_or_offchain(origin.clone())
				.map_err(|_| Error::<T>::DeniedOperation)?;
			

			// First check if claim has already been processed
			let mut snapshot = Self::validate_unclaimed(&icon_address,&ice_address,&server_response)?;
			
			let amount = server_response.get_total_balance::<T>()?;
			Self::validate_creditor_fund(amount)?;
			// validate that the icon signature is valid
			<types::AccountIdOf<T> as Into<<T as Config>::VerifiableAccountId>>::into(
				ice_address.clone(),
			)
			.verify_with_icon(&icon_address, &icon_signature, &message)
			.map_err(|err| {
				log::trace!(
					"[Airdrop pallet] Address pair: {:?} was ignored. {}{:?}",
					(&ice_address, &icon_address),
					"Signature verification failed with error: ",
					err
				);
				Error::<T>::InvalidSignature
			})?;

			Self::do_transfer( &server_response,&mut snapshot)?;

			Self::deposit_event(Event::<T>::ClaimSuccess(icon_address.clone()));

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
			ice_address: types::AccountIdOf<T>,
			server_response: types::ServerResponse,
		) -> DispatchResultWithPostInfo {
			// Make sure its callable by sudo or offchain
			ensure_root(origin.clone()).map_err(|_| Error::<T>::DeniedOperation)?;

			

			// First check if claim has already been processed
			let mut snapshot = Self::validate_unclaimed(&icon_address,&ice_address,&server_response)?;
			
			// check creditor balance
			let amount = server_response.get_total_balance::<T>()?;
			Self::validate_creditor_fund(amount)?;

			Self::do_transfer(&server_response,&mut snapshot)?;

			Self::deposit_event(Event::<T>::ClaimSuccess(icon_address.clone()));

			Ok(Pays::No.into())
		}

		/// Dispatchable to be called by user when they want to
		/// make a claim request
		//
		// TODO:
		// Now, we are checking the validation inside the dispatchable
		// however substrate provide a way to filter the extrinsic call
		// even before they are kept in transaction pool
		// so if we can do that, we don't have to check for signature_validation
		// here ( we will be checking that before putting the call to pool )
		// this will filter the invalid call before that are kept in pool so
		// allowing valid transaction to take over which inturn improve
		// node performance
		#[pallet::weight(<T as Config>::AirdropWeightInfo::claim_request())]
		pub fn claim_request(
			origin: OriginFor<T>,
			icon_address: types::IconAddress,
			message: Vec<u8>,
			icon_signature: types::IconSignature,
		) -> DispatchResult {
			// Take signed address compatible with airdrop_pallet::Config::AccountId type
			// so that we can call verify_with_icon method

			let ice_address: types::AccountIdOf<T> = ensure_signed(origin)?.into();

			// Check that we are ok to receive new claim request
			if Self::get_airdrop_state().block_claim_request {
				return Err(Error::<T>::NewClaimRequestBlocked.into());
			}

			log::trace!(
				"[Airdorp pallet] Claim_request called by user: {:?} with message: {:?} and sig: {:?}",
				(&icon_address, &ice_address),
				message, icon_signature
			);

			// make sure the validation is correct
			<types::AccountIdOf<T> as Into<<T as Config>::VerifiableAccountId>>::into(ice_address.clone())
				.verify_with_icon(&icon_address, &icon_signature, &message)
				.map_err(|err| {
					log::trace!(
						"[Airdrop pallet] Address pair: {:?} was ignored. {}{:?}",
						(&ice_address, &icon_address),
						"Signature verification failed with error: ",
						err
					);
					Error::<T>::InvalidSignature
				})?;

			Self::claim_request_unverified(ice_address, icon_address)
		}

		// Means to push claim request force fully
		// This skips signature verification
		#[pallet::weight(<T as Config>::AirdropWeightInfo::force_claim_request(0u32))]
		pub fn force_claim_request(
			origin: OriginFor<T>,
			ice_address: types::AccountIdOf<T>,
			icon_address: types::IconAddress,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin).map_err(|_| Error::<T>::DeniedOperation)?;

			log::trace!(
				"[Airdrop pallet] Address pair: {:?} was forced to be inserted",
				(&ice_address, &icon_address),
			);

			Self::claim_request_unverified(ice_address, icon_address)?;

			Ok(Pays::No.into())
		}

		/// Dispatchable to transfer the fund from system balance to given address
		/// As this transfer the system balance, this must only be called within
		/// sudo or with root origin
		// TODO:
		// Sudo pallet can have only one key at a time. This implies that
		// when sudo key is changed we also have to insert the new sudo key in the
		// keystore and rotate it so that offchain worker always use new sudo account.
		// failing to do so will bring offchain worker to a state where it can't send
		// authorised signed transaction anymore leaving all call to this function to
		// fail
		// Some solution:
		// 1) Node operator should ensure to sync sudo key and keystore as above
		// 2) Maintain list of accounts local to this pallet only, (was already done)
		//	  This seems to be good to go approach but brings about the hazard &
		//	  confusion to maintain two list of authorised accounts
		//
		// If any of the step fails in this function,
		// we pass the flow to register_failed_claim if needed to be retry again
		// and cancel the request if it dont have to retried again
		#[pallet::weight(<T as Config>::AirdropWeightInfo::complete_transfer(
			types::block_number_to_u32::<T>(block_number.clone()),
			receiver_icon.len() as u32)
		)]
		pub fn complete_transfer(
			origin: OriginFor<T>,
			block_number: types::BlockNumberOf<T>,
			receiver_icon: types::IconAddress,
			server_response: types::ServerResponse,
		) -> DispatchResultWithPostInfo {
			// Make sure this is either sudo or root
			Self::ensure_root_or_offchain(origin.clone())
				.map_err(|_| Error::<T>::DeniedOperation)?;

			// Make sure chain is ready to process new request
			if Self::get_airdrop_state().avoid_claim_processing {
				log::info!(
					"[Airdrop pallet] Called complete_transfer for {:?} but avoided.",
					receiver_icon
				);
				return Err(Error::<T>::ClaimProcessingBlocked.into());
			}

			// Check again if it is still in the pending queue
			// Eg: If another node had processed the same request
			// or if user had decided to cancel_claim_request
			// this entry won't be present in the queue
			let is_in_queue = <PendingClaims<T>>::contains_key(&block_number, &receiver_icon);
			ensure!(is_in_queue, {
				log::info!(
					"[Airdrop pallet] There is no entry in PendingClaims for {:?}",
					(receiver_icon, block_number)
				);
				Error::<T>::NotInQueue
			});

			// Get snapshot from map and return with error if not present
			let mut snapshot = Self::get_icon_snapshot_map(&receiver_icon)
				.ok_or(())
				.map_err(|_| {
					log::info!(
						"[Airdrop pallet] There is no entry in SnapshotMap for {:?}",
						receiver_icon
					);
					Error::<T>::IncompleteData
				})?;

			// If both of transfer is done. return early
			if snapshot.done_vesting && snapshot.done_instant {
				return Err(Error::<T>::ClaimAlreadyMade.into());
			}

			// Apply all vesting
			Self::do_transfer(&server_response, &mut snapshot).map_err(|_| {
				Self::register_failed_claim(
					frame_system::RawOrigin::Root.into(),
					block_number,
					receiver_icon.clone(),
				)
				.expect("Calling register_failed_claim should not have been failed...");

				Error::<T>::CantApplyVesting
			})?;

			<IceSnapshotMap<T>>::insert(&receiver_icon, snapshot);

			// Now we can remove this claim from queue
			<PendingClaims<T>>::remove(&block_number, &receiver_icon);

			log::trace!(
				"[Airdrop pallet] Transfer done for pair {:?}",
				(&receiver_icon, &block_number)
			);

			Self::deposit_event(Event::<T>::ClaimSuccess(receiver_icon));

			Ok(Pays::No.into())
		}

		/// Call to set OffchainWorker Account ( Restricted to root only )
		/// Why?
		/// We have to have a way that this signed call is from offchain so we can perform
		/// critical operation. When offchain worker key and this storage have same account
		/// then we have a way to ensure this call is from offchain worker
		#[pallet::weight(<T as Config>::AirdropWeightInfo::set_offchain_account())]
		pub fn set_offchain_account(
			origin: OriginFor<T>,
			new_account: types::AccountIdOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin).map_err(|_| Error::<T>::DeniedOperation)?;

			let old_account = Self::get_offchain_account();
			<OffchainAccount<T>>::set(Some(new_account.clone()));

			log::info!(
				"[Airdrop pallet] {} {:?}",
				"Value for OffchainAccount was changed in onchain storage. (Old, New): ",
				(&old_account, &new_account)
			);

			Self::deposit_event(Event::OffchainAccountChanged {
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

		#[pallet::weight(<T as Config>::AirdropWeightInfo::remove_from_pending_queue(
			types::block_number_to_u32::<T>(block_number.clone()),
			icon_address.len() as u32)
		)]
		pub fn remove_from_pending_queue(
			origin: OriginFor<T>,
			block_number: types::BlockNumberOf<T>,
			icon_address: types::IconAddress,
		) -> DispatchResultWithPostInfo {
			Self::ensure_root_or_offchain(origin)?;

			<PendingClaims<T>>::remove(&block_number, &icon_address);
			Self::deposit_event(Event::<T>::RemovedFromQueue(icon_address));

			log::info!(
				"[Airdrop pallet] {} {:?}",
				"Remove_from_pending_queue called for {:?}. Pass",
				(&icon_address, &block_number)
			);

			Ok(Pays::No.into())
		}

		/// Call that handles what to do when an entry failed while
		/// processing in offchain worker
		/// We move the entry to future block key so that another
		/// offchain worker can process it again

		#[pallet::weight(<T as Config>::AirdropWeightInfo::register_failed_claim(
			types::block_number_to_u32::<T>(block_number.clone()),
			icon_address.len() as u32)
		)]
		pub fn register_failed_claim(
			origin: OriginFor<T>,
			block_number: types::BlockNumberOf<T>,
			icon_address: types::IconAddress,
		) -> DispatchResultWithPostInfo {
			Self::ensure_root_or_offchain(origin).map_err(|_| Error::<T>::DeniedOperation)?;

			let ice_address = Self::get_icon_snapshot_map(&icon_address)
				.ok_or(())
				.map_err(|_| {
					log::info!(
						"[Airdrop pallet] {}. {:?}",
						"Cannot register as failed claim because was not in map",
						(&icon_address, &block_number)
					);
					Error::<T>::IncompleteData
				})?
				.ice_address;
			let retry_remaining = Self::get_pending_claims(&block_number, &icon_address)
				.ok_or(())
				.map_err(|_| {
					log::info!(
						"[Airdrop pallet] {}. {:?}",
						"Cannot register as failed claim because was not in queue",
						(&icon_address, &block_number)
					);
					Error::<T>::NotInQueue
				})?;

			// In both case weather retry is remaining or not
			// we will remove this entry from this block number key
			// so do it early to make sure we dont miss this inany case
			// otherwise this entry will always be floating around in storage
			<PendingClaims<T>>::remove(&block_number, &icon_address);

			// Check if it's retry counter have been brought to zero
			// if so do not move this entry to next block
			if retry_remaining == 0_u8 {
				log::trace!(
					"[Airdrop pallet] Retry limit exceed for pair {:?} is in block number: {:?}",
					(&ice_address, &icon_address),
					block_number
				);

				// TODO:
				// What to do when retry exceed?
				// it is already removed from queue in previous statements
				// so is there nothing else to do?

				// Emit event that retry been exceed
				Self::deposit_event(Event::<T>::RetryExceed {
					ice_address,
					icon_address,
					was_in: block_number,
				});

				// Exceeding retry is still an expected Ok behaviour
				return Ok(Pays::No.into());
			}

			// This entry have some retry remaining so we put this entry in another block_number key
			let new_block_number = Self::get_current_block_number().saturating_add(1_u32.into());
			<PendingClaims<T>>::insert(
				&new_block_number,
				&icon_address,
				retry_remaining.saturating_sub(1),
			);

			log::trace!(
				"[Airdrop pallet] Register of pair {:?} in height {:?} succeed. Old retry: ",
				(&ice_address, &icon_address),
				new_block_number
			);
			Self::deposit_event(Event::<T>::RegisteredFailedClaim(
				ice_address.clone(),
				new_block_number,
				retry_remaining,
			));
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

		#[pallet::weight(<T as Config>::AirdropWeightInfo::update_processed_upto_counter(
			types::block_number_to_u32::<T>(new_value.clone()))
		)]
		pub fn update_processed_upto_counter(
			origin: OriginFor<T>,
			new_value: types::BlockNumberOf<T>,
		) -> DispatchResultWithPostInfo {
			Self::ensure_root_or_offchain(origin).map(|_| Error::<T>::DeniedOperation)?;

			log::trace!("ProceedUpto Counter updating to value: {:?}", new_value);
			<ProcessedUpto<T>>::set(new_value);

			Self::deposit_event(Event::<T>::ProcessedCounterSet(new_value));

			Ok(Pays::No.into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: types::BlockNumberOf<T>) {
			// If this is not the block to start offchain worker
			// print a log and early return
			if !Self::should_run_on_this_block(block_number) {
				log::trace!(
					"[Airdrop pallet] Offchain worker skipped for block: {:?}",
					block_number
				);
				return;
			}

			// If this network is no longer processing claim request
			if Self::get_airdrop_state().avoid_claim_processing {
				log::trace!(
					"[Airdrop pallet] Offchain worker avoided for block {:?}",
					block_number
				);
			}

			log::info!(
				"Creditor account address is: {:?}",
				Self::get_creditor_account()
			);

			// Start processing from one + the block previously processed
			let start_processing_from =
				Self::get_processed_upto_counter().saturating_add(1_u32.into());

			// Mark the block numbers we will be processing
			let mut blocks_to_process =
				types::PendingClaimsOf::<T>::new(start_processing_from..block_number);

			while let Some((block_number, mut entries_in_block)) = blocks_to_process.next() {
				while let Some(claimer) = entries_in_block.next() {
					let claim_res = Self::process_claim_request((block_number, claimer.clone()));

					if let Err(err) = claim_res {
						log::error!(
							"Claim process failed for Pending entry {:?} with error {:?}",
							(claimer, block_number),
							err
						);
					} else {
						log::trace!(
							"complete_transfer function called sucessfully on Pending entry {:?}",
							(claimer, block_number)
						);
					}
				}
			}

			let update_res = Self::make_signed_call(&Call::update_processed_upto_counter {
				new_value: block_number.saturating_sub(1_u32.into()),
			});

			// We assume that calling extrinsic will always pass. If not here is worst case scanerio
			// update counter is never updated i.e always remains 0
			// and we will always try processing claims made from request
			// 0..10(no of block to process in one ocw)
			// So:
			// something should keep checking for process_upto value and notify when
			// the gap between this value and current block is too high
			// ( which signifies that this counter is not being updated lately )
			//
			// Just ignoring the result
			update_res.ok();
		}
	}

	// implement all the helper function that are called from pallet hooks like offchain_worker
	impl<T: Config> Pallet<T> {
		// Function to proceed single claim request
		pub fn process_claim_request(
			(stored_block_num, icon_address): (types::BlockNumberOf<T>, types::IconAddress),
		) -> Result<(), types::ClaimError> {
			use types::{ClaimError, ServerError};

			let server_response_res = Self::fetch_from_server(&icon_address);

			log::trace!(
				"[Airdrop pallet] Server returned data for {:?} is {:#?}",
				icon_address,
				server_response_res
			);

			let call_to_make: Call<T>;
			match server_response_res {
				// If error is NonExistentData then, it signifies this icon address do not exists in server
				// so we just cancel the request
				Err(ClaimError::ServerError(ServerError::NonExistentData)) => {
					// TODO: should we call register_fail or remove_fom_queue?
					log::trace!(
						"[Airdrop pallet] Server returned NoData for entry {:?}",
						(&icon_address, &stored_block_num)
					);
					call_to_make = Call::register_failed_claim {
						icon_address: icon_address.clone(),
						block_number: stored_block_num,
					};
				}

				// If is any other error, just propagate it to caller
				Err(err) => {
					log::error!(
						"[Airdrop pallet] Cannot fetch from server for entry {:?}. Error: {:?}",
						(&icon_address, &stored_block_num),
						err
					);
					return Err(err);
				}

				// if response is valid, then call complete_transfer dispatchable
				// This will also clear the queue
				Ok(response) => {
					call_to_make = Call::complete_transfer {
						receiver_icon: icon_address.clone(),
						server_response: response,
						block_number: stored_block_num,
					};
				}
			}

			let call_res = Self::make_signed_call(&call_to_make);
			call_res.map_err(|err| {
				// NOTE:
				// If this call failed, i.e this statement is reached
				// then is is responsibility of node operator to do this claim manually
				// Possible sources of error:
				// - See make_signed_call
				//
				// Additionally, it is checked that onchain storage of sudo is same as in keystore
				// but it is possible that this storage is changed after call is put on transaction pool
				// this will also make the call fail as it will no longer be validated as offchain
				//
				// Workaround:
				// While changing offchain account we must do it in this order:
				// 1) remove old entry from KeyStore
				// 2) Add new entry in Keystore
				// 3) Then only call to modiy OffchainAccount on-chain storage
				// - On step 1,2 `make_signed_call` will fail but at least we wont loose the entry
				// - and after we complete step 3, all will fine

				log::error!(
					"[Airdorp pallet] Calling extrinsic {:#?} failed with error: {:?}",
					call_to_make,
					err
				);
				ClaimError::CallingError(err)
			})
		}

		/// Helper function to send signed transaction to provided callback
		/// and map the resulting error
		/// @return:
		/// - Error or the account from which transaction was made
		/// NOTE:
		/// As all call are sent from here, it is important to verify that ::any_account()
		/// in this context returns accounts that are authorised
		/// i.e present in Storage::SudoAccount
		/// So that the call is not discarded inside the calling function if it checks for so
		/// This can be done by intentionally if done:
		/// rpc call to: author.insertKey(crate::KEY_TYPE_ID, _, ACCOUNT_X)
		/// extrensic call to: Self::set_authorised(ACCOUNT_X) // Same as above account
		pub fn make_signed_call(
			call_to_make: &Call<T>,
		) -> Result<(), types::CallDispatchableError> {
			use frame_system::offchain::SendSignedTransaction;
			use types::CallDispatchableError;

			let signer = frame_system::offchain::Signer::<T, T::AuthorityId>::any_account();
			let send_tx_res =
				signer.send_signed_transaction(move |_account| (*call_to_make).clone());

			send_tx_res
				.ok_or(CallDispatchableError::NoAccount)?
				.1
				.map_err(|_| CallDispatchableError::CantDispatch)
		}

		/// This function fetch the data from server and return it in required struct
		pub fn fetch_from_server(
			icon_address: &types::IconAddress,
		) -> Result<types::ServerResponse, types::ClaimError> {
			use codec::alloc::string::String;
			use sp_runtime::offchain::{http, Duration};
			use types::ClaimError;

			const FETCH_TIMEOUT_PERIOD_MS: u64 = 4_0000;
			let timeout =
				sp_io::offchain::timestamp().add(Duration::from_millis(FETCH_TIMEOUT_PERIOD_MS));

			let request_url = String::from_utf8(
				T::FetchIconEndpoint::get()
					.as_bytes()
					.iter()
					// always prefix the icon_address with 0x
					.chain(b"0x")
					// we have to first bring icon_address in hex format
					.chain(hex::encode(icon_address).as_bytes())
					.cloned()
					.collect(),
			)
			.map_err(|err| {
				log::error!(
					"[Airdrop pallet] While encoding http url for {:?}. Error: {:?}",
					icon_address,
					err
				);
				ClaimError::HttpError
			})?;

			log::trace!("[Airdrop pallet] Sending request to {}", request_url);
			let request = http::Request::get(request_url.as_str());

			let pending = request.deadline(timeout).send().map_err(|e| {
				log::warn!("[Airdrop pallet]. Error in {} : {:?}", line!(), e);
				ClaimError::HttpError
			})?;

			log::trace!("[Airdrop pallet] Initilizing response variable..");
			let response = pending
				.try_wait(timeout)
				.map_err(|e| {
					log::warn!("[Airdrop pallet]. Error in {} : {:?}", line!(), e);
					ClaimError::HttpError
				})?
				.map_err(|e| {
					log::warn!("[Airdrop pallet]. Error in {} : {:?}", line!(), e);
					ClaimError::HttpError
				})?;

			// try to get the response bytes if server returned 200 code
			// or just return early
			let response_bytes: Vec<u8>;
			if response.code == 200 {
				response_bytes = response.body().collect();
			} else {
				log::warn!(
					"[Airdrop pallet]. Error in {}. Unexpected http code: {}",
					line!(),
					response.code
				);
				return Err(ClaimError::HttpError);
			}

			utils::unpack_server_response(response_bytes.as_slice())
		}

		/// Return an indicater (bool) on weather the offchain worker
		/// should be run on this block number or not
		pub fn should_run_on_this_block(block_number: types::BlockNumberOf<T>) -> bool {
			block_number % crate::OFFCHAIN_WORKER_BLOCK_GAP.into() == 0_u32.into()
		}

		// /// Returns the amount to be transferred from given serverResponse
		// pub fn get_total_amount(
		// 	server_response: &types::ServerResponse,
		// ) -> Result<types::BalanceOf<T>, &'static str> {
		// 	use sp_runtime::traits::CheckedAdd;

		// 	let amount: types::BalanceOf<T> = server_response
		// 		.amount
		// 		.try_into()
		// 		.map_err(|_| "Failed to convert server_response.amount to balance type")?;
		// 	let stake: types::BalanceOf<T> = server_response
		// 		.stake
		// 		.try_into()
		// 		.map_err(|_| "Failed to convert server_response.stake to balance type")?;
		// 	let omm: types::BalanceOf<T> = server_response
		// 		.omm
		// 		.try_into()
		// 		.map_err(|_| "Failed to convert server_response.oom to balance type")?;

		// 	amount
		// 		.checked_add(&stake)
		// 		.ok_or("Adding server_response(amount+stake) overflowed")?
		// 		.checked_add(&omm)
		// 		.ok_or("Adding server_response(amount+stake+omm) overflowed")
		// }
	}

	// implement all the helper function that are called from pallet dispatchable
	impl<T: Config> Pallet<T> {
		pub fn get_creditor_account() -> types::AccountIdOf<T> {
			use sp_runtime::traits::AccountIdConversion;

			T::Creditor::get().into_account()
		}

		/// Do claim request withing checking for anything.
		/// This is seperated as a means to share logic
		/// And always should be called only after doing proper check before hand
		pub fn claim_request_unverified(
			ice_address: types::AccountIdOf<T>,
			icon_address: types::IconAddress,
		) -> DispatchResult {
			// We check the claim status before hand
			let is_already_on_map = <IceSnapshotMap<T>>::contains_key(&icon_address);
			ensure!(!is_already_on_map, {
				log::trace!(
					"[Airdrop pallet] Address pair: {:?} was ignored. {}",
					(&ice_address, &icon_address),
					"Entry already exists in map"
				);
				Error::<T>::RequestAlreadyMade
			});

			// Get the current block number. This is the number where user asked for claim
			// and we store it in PencingClaims to preserve FIFO
			let current_block_number = Self::get_current_block_number();

			// Insert with default snapshot but with real icon address mapping
			let new_snapshot = types::SnapshotInfo::<T>::default().ice_address(ice_address.clone());
			<IceSnapshotMap<T>>::insert(&icon_address, &new_snapshot);

			// insert in queue respective to current block number
			// and retry as defined in crate level constant
			<PendingClaims<T>>::insert(
				&current_block_number,
				&icon_address,
				crate::DEFAULT_RETRY_COUNT,
			);

			log::trace!(
				"[Airdrop pallet] Address pair: {:?} was inserted in map & pending claims in block height: {:?}",
				(&ice_address, &icon_address),
				current_block_number
			);

			Self::deposit_event(Event::<T>::ClaimRequestSucced {
				registered_in: current_block_number,
				ice_address,
				icon_address,
			});

			Ok(())
		}

		/// Helper function to create similar interface like `ensure_root`
		/// but which instead check for sudo key
		pub fn ensure_root_or_offchain(origin: OriginFor<T>) -> DispatchResult {
			let is_root = ensure_root(origin.clone()).is_ok();
			let is_offchain = {
				let signed = ensure_signed(origin);
				signed.is_ok() && signed.ok() == Self::get_offchain_account()
			};

			ensure!(is_root || is_offchain, DispatchError::BadOrigin);
			Ok(())
		}

		/// Return block height of Node from which this was called
		pub fn get_current_block_number() -> types::BlockNumberOf<T> {
			<frame_system::Pallet<T>>::block_number()
		}

		pub fn validate_unclaimed(icon_address: &types::IconAddress,ice_address: &types::AccountIdOf<T>, server_response:&types::ServerResponse) -> Result<types::SnapshotInfo<T>, Error<T>> {
			 let snapshot= Self::get_icon_snapshot_map(icon_address);
			 if let Some(saved) =snapshot {
				if saved.done_vesting && saved.done_instant {
					return Err(Error::<T>::ClaimAlreadyMade.into());
				}
			 }
			let mut new_snapshot = types::SnapshotInfo::<T>::default().ice_address(ice_address.clone());
			let amount = server_response.get_total_balance::<T>()?;
			new_snapshot.defi_user = server_response.defi_user;
			new_snapshot.amount = amount;

			<IceSnapshotMap<T>>::insert(&icon_address, &new_snapshot);

			 Ok(new_snapshot)
		}

		pub fn validate_creditor_fund(amount: types::BalanceOf<T>) -> Result<(), Error<T>> {
			use sp_std::cmp::Ordering;
			let creditor_balance = <T as Config>::Currency::free_balance(&Self::get_creditor_account());
			let ordering = creditor_balance.cmp(&amount);
			if let Ordering::Less = ordering {
				Self::deposit_event(Event::<T>::CreditorBalanceLow);
				return Err(Error::<T>::InsufficientCreditorBalance);
			}
			Ok(())
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

		pub fn do_transfer(
			server_response: &types::ServerResponse,
			snapshot: &mut types::SnapshotInfo<T>,
		) -> Result<(), DispatchError> {
			// TODO: put more relaible value
			const BLOCKS_IN_YEAR: u32 = 10_000u32;
			// Block number after which enable to do vesting
			const VESTING_APPLICABLE_FROM: u32 = 100u32;

			let claimer = snapshot.ice_address.clone();
			let creditor = Self::get_creditor_account();

			let total_amount = server_response.get_total()?;

			let (mut instant_amount, vesting_amount) =
				Self::get_splitted_amounts(total_amount, server_response.defi_user)?;

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
			let claimer_origin =
				<T::Lookup as sp_runtime::traits::StaticLookup>::unlookup(claimer.clone());

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
					&claimer,
					instant_amount,
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
		pub fn init_balance(account: &types::AccountIdOf<T>, free: u32) {
			T::Currency::make_free_balance_be(account, free.into());
		}

		pub fn setup_claimer(
			claimer: types::AccountIdOf<T>,
			bl_number: types::BlockNumberOf<T>,
			icon_address: types::IconAddress,
		) {
			T::Currency::make_free_balance_be(&claimer, 10_00_00_00u32.into());

			let mut snapshot = types::SnapshotInfo::<T>::default();

			snapshot = snapshot.ice_address(claimer.clone());

			<IceSnapshotMap<T>>::insert(&icon_address, snapshot);

			<PendingClaims<T>>::insert(bl_number, &icon_address, 2_u8);
		}
	}

	
}

pub mod airdrop_crypto {
	use crate::KEY_TYPE_ID;
	use codec::alloc::string::String;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE_ID);

	pub struct AuthId;

	// implemented for runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}
