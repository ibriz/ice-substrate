use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn remove_from_pending_queue(_b: u32, u: u32) -> Weight;
	fn complete_transfer(_b: u32, u: u32) -> Weight;
	fn donate_to_creditor(x: u32) -> Weight;
	fn register_failed_claim(_b: u32, _u: u32) -> Weight;
	fn claim_request() -> Weight;
	fn update_processed_upto_counter(_b: u32) -> Weight;
	fn set_offchain_account() -> Weight;
	fn force_claim_request(_u: u32, ) -> Weight;
	fn dispatch_user_claim()->Weight;
	fn dispatch_exchange_claim()->Weight;
}

/// Weight functions for `pallet_airdrop`.
pub struct AirDropWeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for AirDropWeightInfo<T> {
	// Storage: Airdrop PendingClaims (r:0 w:1)
	fn remove_from_pending_queue(_b: u32, _u: u32) -> Weight {
		(990_312_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Airdrop PendingClaims (r:1 w:1)
	// Storage: Airdrop IceSnapshotMap (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn complete_transfer(_b: u32, u: u32) -> Weight {
		(195_552_000 as Weight)
			// Standard Error: 121_000
			.saturating_add((222_000 as Weight).saturating_mul(u as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: System Account (r:1 w:1)
	fn donate_to_creditor(x: u32) -> Weight {
		(104_548_000 as Weight)
			// Standard Error: 87_000
			.saturating_add((367_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Airdrop IceSnapshotMap (r:1 w:0)
	// Storage: Airdrop PendingClaims (r:1 w:2)
	fn register_failed_claim(_b: u32, _u: u32) -> Weight {
		(109_852_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Airdrop IceSnapshotMap (r:1 w:1)
	// Storage: Airdrop PendingClaims (r:0 w:1)
	fn claim_request() -> Weight {
		(535_200_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Airdrop ProcessedUpto (r:0 w:1)
	fn update_processed_upto_counter(_b: u32) -> Weight {
		(43_555_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Airdrop OffchainAccount (r:1 w:1)
	fn set_offchain_account() -> Weight {
		(568_200_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	// Storage: Airdrop IceSnapshotMap (r:1 w:1)
	// Storage: Airdrop PendingClaims (r:0 w:1)
	fn force_claim_request(_u: u32) -> Weight {
		(107_069_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}

	fn dispatch_user_claim()->Weight{
		return 10000 as Weight;
	}

	fn dispatch_exchange_claim()->Weight{
		return 10000 as Weight;
	}

	
}
