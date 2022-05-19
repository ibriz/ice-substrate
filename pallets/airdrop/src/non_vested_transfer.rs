use crate as airdrop;
use airdrop::{types, utils, Pallet as AirdropModule};
use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement};
use sp_runtime::traits::{CheckedAdd, Convert};

pub struct AllInstantTransfer;
impl types::DoTransfer for AllInstantTransfer {
	fn do_transfer<T: airdrop::Config>(
		snapshot: &mut types::SnapshotInfo<T>,
		icon_address: &types::IconAddress,
		total_balance: types::BalanceOf<T>,
		_defi_user: bool,
	) -> Result<(), DispatchError> {
		let creditor = AirdropModule::<T>::get_creditor_account();
		let claimer = AirdropModule::<T>::to_account_id(snapshot.ice_address)?;

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
