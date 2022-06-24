use crate as airdrop;
use airdrop::{types, Pallet as AirdropModule};
use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement};

#[deprecated(
	note = "Instead of using two different modules to seperate (non-)vesting.
Configure pallet_airdrop::Config::AirdropVariables::*instant_percentage"
)]
pub struct AllInstantTransfer;

impl types::DoTransfer for AllInstantTransfer {
	fn do_transfer<T: airdrop::Config>(
		snapshot: &mut types::SnapshotInfo<T>,
	) -> Result<(), DispatchError> {
		let creditor = AirdropModule::<T>::get_creditor_account();
		let claimer = &snapshot.ice_address;
		let total_balance = snapshot.amount;

		if !snapshot.done_instant {
			<T as airdrop::Config>::Currency::transfer(
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
