use crate as airdrop;
use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement};
use airdrop::{types, utils, Pallet as AirdropModule};
use sp_runtime::traits::{CheckedAdd, Convert};

pub struct DOVestdTransfer;
impl types::DoTransfer for DOVestdTransfer {
	fn do_transfer<T: airdrop::Config>(
		snapshot: &mut types::SnapshotInfo<T>,
		icon_address: &types::IconAddress,
		total_amount: types::ServerBalance,
		defi_user: bool,
	) -> Result<(), DispatchError> {
		// TODO: put more relaible value
		const BLOCKS_IN_YEAR: u32 = 5_256_000u32;
		// Block number after which enable to do vesting
		const VESTING_APPLICABLE_FROM: u32 = 1u32;

		let claimer = snapshot.ice_address;
		let creditor = AirdropModule::<T>::get_creditor_account();

		let (mut instant_amount, vesting_amount) =
			AirdropModule::<T>::get_splitted_amounts(total_amount, defi_user)?;

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
		let claimer_account = AirdropModule::<T>::to_account_id(claimer)?;
		let claimer_origin =
			<T::Lookup as sp_runtime::traits::StaticLookup>::unlookup(claimer_account.clone());

		match transfer_shcedule {
			// Apply vesting
			Some(schedule) if !snapshot.done_vesting => {
				let vest_res = pallet_vesting::Pallet::<T>::vested_transfer(
					creditor_origin,
					claimer_origin,
					schedule,
				);

				match vest_res {
					// Everything went ok. update flag
					Ok(()) => {
						snapshot.done_vesting = true;
						snapshot.vesting_block_number =
							Some(AirdropModule::<T>::get_current_block_number());
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
					AirdropModule::<T>::get_current_block_number()
				);
			}

			// No schedule was created
			None => {
				// If vesting is not applicable once then with same total_amount
				// it will not be applicable ever. So mark it as done.
				snapshot.done_vesting = true;
				snapshot.vesting_block_number =
					Some(AirdropModule::<T>::get_current_block_number());

				log::trace!(
					"[Airdrop pallet] Primary vesting not applicable for {:?}",
					claimer_origin,
				);
			}
		}

		// if not done previously
		// Transfer the amount user is expected to receiver instantly
		if !snapshot.done_instant {
			<T as airdrop::Config>::Currency::transfer(
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
		} else {
			log::trace!(
				"[Airdrop pallet] Doing instant transfer for {:?} skipped in {:?}",
				claimer,
				AirdropModule::<T>::get_current_block_number()
			);
		}

		Ok(())
	}
}
