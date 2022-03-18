
//! Autogenerated weights for `pallet_airdrop`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-18, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// ./


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
	fn remove_from_pending_queue(_x: u32, _b: u32, ) -> Weight {
		(67_877_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
