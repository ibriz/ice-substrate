use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
    fn remove_from_pending_queue() -> Weight;
}




/// Weight functions for `pallet_airdrop`.
pub struct AirDropWeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for AirDropWeightInfo<T> {
	// Storage: Sudo Key (r:1 w:0)
	// Storage: Airdrop PendingClaims (r:0 w:1)
	fn remove_from_pending_queue() -> Weight {
		(67_877_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
