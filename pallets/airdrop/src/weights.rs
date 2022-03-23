use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
    fn remove_from_pending_queue(_b: u32,) -> Weight;
	fn complete_transfer_success(_b: u32, ) -> Weight;
	fn donate_to_creditor(x: u32, ) -> Weight;
	fn register_failed_claim(_b: u32, ) -> Weight;
	fn claim_request() -> Weight;
}




/// Weight functions for `pallet_airdrop`.
pub struct AirDropWeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for AirDropWeightInfo<T> {
	// Storage: Sudo Key (r:1 w:0)
	// Storage: Airdrop PendingClaims (r:0 w:1)
	fn remove_from_pending_queue(_b: u32, ) -> Weight {
		(71_233_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Sudo Key (r:1 w:0)
	// Storage: Airdrop PendingClaims (r:1 w:1)
	// Storage: Airdrop IceSnapshotMap (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	fn complete_transfer_success(_b: u32, ) -> Weight {
		(674_320_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: System Account (r:1 w:1)
	fn donate_to_creditor(x: u32, ) -> Weight {
		(113_199_000 as Weight)
			// Standard Error: 105_000
			.saturating_add((70_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Sudo Key (r:1 w:0)
	// Storage: Airdrop PendingClaims (r:1 w:2)
	fn register_failed_claim(_b: u32, ) -> Weight {
		(68_980_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Airdrop IceSnapshotMap (r:1 w:1)
	// Storage: Airdrop PendingClaims (r:0 w:1)
	fn claim_request() -> Weight {
		(495_400_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}

	
}
