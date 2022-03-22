
//! Autogenerated weights for `pallet_airdrop`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-22, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// F:\projects\ice-substrate\target\release\ice-node.exe
// benchmark
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_airdrop
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --raw=raw.json
// --output
// ./airdrop_weights.rs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_airdrop`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_airdrop::WeightInfo for WeightInfo<T> {
	// Storage: Sudo Key (r:1 w:0)
	// Storage: Airdrop PendingClaims (r:0 w:1)
	fn remove_from_pending_queue(x: u32, _b: u32, ) -> Weight {
		(51_484_000 as Weight)
			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Sudo Key (r:1 w:0)
	// Storage: Airdrop PendingClaims (r:1 w:1)
	// Storage: Airdrop IceSnapshotMap (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	fn complete_transfer_success(_x: u32, _b: u32, c: u32, ) -> Weight {
		(682_130_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((1_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: System Account (r:1 w:1)
	fn donate_to_creditor(_x: u32, ) -> Weight {
		(150_184_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
