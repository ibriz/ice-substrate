//! Benchmarking setup for pallet-airdrop

use super::*;

use crate as pallet_airdrop;
#[allow(unused)]
use crate::Pallet;
use codec::Decode;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{pallet_prelude::*, traits::ConstU32, BoundedVec};
use frame_system::RawOrigin;
use sp_core::sr25519;
use sp_core::*;
use sp_runtime::traits::Convert;
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::traits::Saturating;
use sp_std::prelude::*;
use types::{AccountIdOf, BlockNumberOf, IceAddress, MerkleHash, RawPayload};

#[derive(Clone, Debug)]
pub struct BenchmarkSample<'a> {
	pub icon_address: &'a str,
	pub ice_address: &'a str,
	pub message: &'a str,
	pub icon_signature: &'a str,
	pub ice_signature: &'a str,
	pub amount: u128,
	pub defi_user: bool,
	pub merkle_proofs: &'a [&'a str],
	pub merkle_root: &'a str,
}

#[derive(Clone, Debug)]
pub struct UserClaimTestCase<T: Get<u32>> {
	pub icon_address: types::IconAddress,
	pub ice_address: IceAddress,
	pub message: RawPayload,
	pub icon_signature: types::IconSignature,
	pub ice_signature: types::IceSignature,
	pub amount: u128,
	pub defi_user: bool,
	pub merkle_proofs: BoundedVec<MerkleHash, T>,
	pub merkle_root: [u8; 32],
}

impl<'a, B> From<BenchmarkSample<'a>> for UserClaimTestCase<B>
where
	B: Get<u32>,
{
	fn from(sample: BenchmarkSample<'a>) -> Self {
		let proff_size = B::get();

		assert_eq!(sample.merkle_proofs.len(), proff_size as usize);
		let amount = sample.amount;
		let defi_user = sample.defi_user;
		let ice_address = hex::decode(sample.ice_address).unwrap().try_into().unwrap();
		let merkle_root = hex::decode(sample.merkle_root).unwrap().try_into().unwrap();
		let message = sample.message.as_bytes().to_vec().try_into().unwrap();
		let icon_address = hex::decode(sample.icon_address)
			.unwrap()
			.try_into()
			.unwrap();
		let icon_signature = hex::decode(sample.icon_signature)
			.unwrap()
			.try_into()
			.unwrap();
		let ice_signature = hex::decode(sample.ice_signature)
			.unwrap()
			.try_into()
			.unwrap();
		let merkle_proofs = sample
			.merkle_proofs
			.iter()
			.map(|proff| {
				let proff = hex::decode(proff).unwrap();
				proff.try_into().unwrap()
			})
			.collect::<Vec<MerkleHash>>()
			.try_into()
			.unwrap();

		UserClaimTestCase::<B> {
			icon_address,
			ice_address,
			message,
			merkle_root,
			icon_signature,
			ice_signature,
			amount,
			defi_user,
			merkle_proofs,
		}
	}
}

pub const benchmark_samples: [BenchmarkSample; 4] = [
	   BenchmarkSample{
		   amount: 8800000000,
		   defi_user: false,
		   ice_address: "14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f",
		   ice_signature: "dc969151b35aa41da2382ec1f1c6fc413b309db4c476fde6ed84b339aad68a48099add8ca23a52c10961568679f49db98d43d95901594d44a04cfdfdc3723685",
		   icon_address: "1abd441b0b42422387a5ff7c5705a529b3cce196",
		   icon_signature: "e37ce5be477ebfe930cf40becc502c76b1e096f262df231ca7a78408d8b8f4e672bf0bbdd8c37df152f4a1c3de23ce2c568c711a3223e8cf3ab20fc1dd37800500",
		   merkle_proofs: &[
			"ab5d9740c85676bda7230cec714ab54a3e667e2fa115b271d5eba07f70044549",
            "eccb38ee0e0e51792917b7b2952664eb490137e111013692f8226adf27bb474d",
            "0846539a1e8bad70f80201eeda1bfe70071e013e7e10d1d624ce9fc9a055d0e8",
            "47eea037694f9d7d7537844686405d4e6e0fe4df7a3eb77f376d4b148108f7bb",
            "ef91c39b6d4d06139492fe994f20540aaaace51a94ebbe39bb4ab6ede3431bcb",
            "419c9d60d9a49d7a40cdf2ab4e86a61d7a691a0695989351e002004d1f44ac4c",
            "510073a91f8ea91babf4fc48c52bd2bb9b996e8e9118d9bed07cb531d712f602",
            "a24c8ed15b8f15c7c74de3419bb84a0af12ab842bd444c08db884da13a49839f",
            "9d114c5f584d3f2db8c7fcd8f73047ef8764a38a5f786e86cae18a419d54900e",
            "0276f3bcf0b3b041ed3fc8d6f2da65d5c7d44b986cef45712455c414e081f127"
		   ],
		   merkle_root: "4e066eee09e3daed2566830b4f422efc6202745f2d643f171e7b582988cfe380",
		   message: "icx_sendTransaction.data.{method.transfer.params.{wallet.14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f}}.dataType.call.from.hx1abd441b0b42422387a5ff7c5705a529b3cce196.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hx1abd441b0b42422387a5ff7c5705a529b3cce196.version.0x3"

	   },
	   BenchmarkSample {
		amount:4800000000,
		defi_user:true,
		ice_address:"14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f",
		ice_signature:"0e95a955b252791cde8ad123fdbd1d9083e2422b320455c1ba0d102d49f93845d7b9e7d7a9b3d88172990cd04160c4f3152ed9fc935b63dfe20838c4530da78c",
		icon_address:"686d9a7dd2daae5ff9fb7d75d9e246d7fc757f7a",
		icon_signature:"6fa8c1e2465b4df352fac6ff5e8f991e40ffdaa735a0b1712ff5dc430410a9be199dc7dbe26f9495143dc2e457d6f3b1461ef863d0c0457077362a21e5e4ef9201",
		merkle_proofs:&[
			"28573640059d895cebff023ffe9bd1d6bc7cd78bf0cba3c6c2bb5407c05f946c",
            "c8abfcb3e2f3068b0ceb2f7da95b0400cbd203d93f463bc541757a550cd50990",
            "68a64b952f27e6d97096c8d9aee45f43c5c0621160a739ca4d64c406bd90e3b6",
            "b276afe29023a35a9c7974b6f077562c3f873ec2cd8e67790bbaf76a06bbdecd",
            "4709274d8bc787f4dadf7b68868bcb48577cb9718478900fdaac24317e95ee44",
            "09af9ca4963ca11c108e42206bc1c5330eaab6861b9f95b3f7541b05c7497028",
            "7896cf14bd6dba1cf0bb455cc078173b727a7a580221fb3abc7f4e496c9b85b6",
            "99b0bfb7fefb2f7ebcb86f32152f686f644b551372d55fbc403610feec2106ae",
            "2b458e4a9fe78597c463e408a615fe745a8849bbf010c8cec39ddde44df55099",
            "3b43a9ace231d9879e9627c59a02ff6d71e0263dfded179d69f5f648174409c2"
		],
		merkle_root:"4e066eee09e3daed2566830b4f422efc6202745f2d643f171e7b582988cfe380",
		message:"icx_sendTransaction.data.{method.transfer.params.{wallet.14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f}}.dataType.call.from.hx686d9a7dd2daae5ff9fb7d75d9e246d7fc757f7a.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hx686d9a7dd2daae5ff9fb7d75d9e246d7fc757f7a.version.0x3"

	},
	BenchmarkSample{
		amount:5600000000,
		defi_user:false,
		ice_address:"14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f",
		ice_signature:"52d2bd42c62a34e091e52f0d371862098c345dac2bf17d7f5cf128721037261921864d82c61025337a4305556365f5d53b64f991aeac5632eb5f80bac1f1ae8b",
		icon_address:"4906b6524a5fce5427c12eeaf32a79e714f65aa4",
		icon_signature:"065d3725813fca25d18aa537f23c5504682a67b7545fd23529866867cf337eb01ff1696da5f90cf447709c6edcdf7db89578ebabecf5f0c180e90efeb0c03a4d00",
		merkle_proofs:&[
			"cfe59aa71f774d0109f3d9d00cfb1307edecdd14a7db28dfd6fee06815c78217",
            "7e71fdf8905c49bda1a2faabaf69e8c48ca1d01fb4690af0da51ecef1f459871",
            "9cc9625dd9057526a704a56e7dc39b5670d601e16dace5c368d5a1b553933683",
            "f2d89692aeb9f2772aeb4430f46969d9701002a0527192b1cfacb2f42a06bba4",
            "f9b0620deac81f373ae5764c5d39097c76c3c44c4a2a0c652450c716c76cfff4",
            "86440e8fb040e6f4c2c6075667a5609f869f461e552dd57a4ba995db0948ef0e",
            "79dab9eb072fccfd645c30ad417711019ccbbe9da5d61e3e9f29048a933317de",
            "37db6f65c555876ee1f4201c85737bffc0ca221d896e2456b2c7cf2d08d883e6",
            "5ba24d44b6ce748ecf4930f447c107461b729fc08d4053b36aed17c63f5f72bd",
            "0276f3bcf0b3b041ed3fc8d6f2da65d5c7d44b986cef45712455c414e081f127"
		],
		merkle_root:"4e066eee09e3daed2566830b4f422efc6202745f2d643f171e7b582988cfe380",
		message:"icx_sendTransaction.data.{method.transfer.params.{wallet.14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f}}.dataType.call.from.hx4906b6524a5fce5427c12eeaf32a79e714f65aa4.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hx4906b6524a5fce5427c12eeaf32a79e714f65aa4.version.0x3"

	},
	BenchmarkSample{
		amount:3000000000,
		defi_user:true,
		ice_address:"14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f",
		ice_signature:"64dbc6a8c772f5e470dd38ada6ed563bf1d119fa7268328310dd4157f086120366b6d175444446316aa8cf9d2f12120d86c5feb82bfe8ea5eb70ed76e3be2a8b",
		icon_address:"83094355827de1c80773361edd06957f06915acc",
		icon_signature:"ff97a8229ea65c253f99e67607a28a52e0fe5ef5e0781669981329035d96e3ff1e5c5dead6f484592429be87356985d6f3f4e5a0a80361a811661cc738498cf600",
		merkle_proofs:&[
			"6d0a0a2de2072733d0710ed5583b701bde4b3c304d64aeddb07aa5f76bff7fca",
            "0f7253b4150927707a49c3f48b65f620acc2744988ec420e74151d92e1493ca2",
            "c8dbe2f0bf3080ab8b8a711d068a1ab204b3336c010e8cce07cd4f244e9619ca",
            "fdbaaa3532e23c637e37929bcd6c5ca42c4c46c68f3a3532b441deed70aa4d04",
            "8c7232a8d7f5d694afb732a0c7530a2282f1099e83702f8cd2a07c15004e419b",
            "fc8009fdecf635e2b051de5bfb2d84c9ed6daf0d890f7c62894be308d6941db3",
            "4f489320b17a0bc4e3568cd1b6aba042ff8afdee9ca30f339ed7435565890c0b",
            "ec17ed1368b31e0bd7fda745d60bac9d9f0c0172c9bab6421e938b75b1fa105e",
            "a43ab42be71c641ca14b8b42e392cffa5bdb1591e1378c6a38c2745b7ae1f3fb",
            "3b43a9ace231d9879e9627c59a02ff6d71e0263dfded179d69f5f648174409c2"
		],
		merkle_root:"4e066eee09e3daed2566830b4f422efc6202745f2d643f171e7b582988cfe380",
		message:"icx_sendTransaction.data.{method.transfer.params.{wallet.14524435eb22c05c20e773cb6298886961d632f3ec29f4e4245b02710da2a22f}}.dataType.call.from.hx83094355827de1c80773361edd06957f06915acc.nid.0x1.nonce.0x1.stepLimit.0x0.timestamp.0x0.to.hx83094355827de1c80773361edd06957f06915acc.version.0x3"

	}
   ];

const creditor_key: sp_core::sr25519::Public = sr25519::Public([1; 32]);

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	set_airdrop_server_account {
				let old_account: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();

				let new_account: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();

				<ServerAccount<T>>::set(Some(old_account.clone()));

			}: set_airdrop_server_account(RawOrigin::Root,new_account.clone())
	verify {
				assert_last_event::<T>(Event::ServerAccountChanged{
					old_account:Some(old_account.clone()),
					new_account:new_account.clone()
				}.into());
	}

	update_airdrop_state {
		let old_state= Pallet::<T>::get_airdrop_state();
		let new_state = types::AirdropState::default();

	}: update_airdrop_state(RawOrigin::Root, new_state.clone())
	verify {
		 assert_last_event::<T>(Event::AirdropStateUpdated {
			old_state,
			new_state,
		}.into());
	}

	change_merkle_root {
		let p in 0u8..50u8;

		let zero_31 = [0u8; 31];
		let new_root = [p, ..zero_31];
		let mut last_root = None;
	}: change_merkle_root(
		RawOrigin::Root,
		new_root
	) verify {
		assert_last_event::<T>(Event::MerkleRootUpdated{
			old_root: last_root,
			new_root,
		});
	}

	dispatch_user_claim {
		let x in 0 .. 3;
		let caller: types::AccountIdOf<T> = frame_benchmarking::whitelisted_caller();
		// let ofw_account = sr25519::Public([1; 32]).into_account();
		Pallet::<T>::set_creditor_account(creditor_key);
		let system_account_id = Pallet::<T>::get_creditor_account();
		Pallet::<T>::init_balance(&system_account_id,10_000_000_000);
		let case= UserClaimTestCase::<<T as pallet::Config>::MaxProofSize>::try_from(benchmark_samples[x as usize].clone()).unwrap();
		let amount = <T::BalanceTypeConversion as Convert<_, _>>::convert(case.amount);
		 let icon_address=case.icon_address.clone();

	}: dispatch_user_claim(
		RawOrigin::Root,
		case.icon_address,
		case.ice_address,
		case.message,
		case.icon_signature,
		case.ice_signature,
		amount,
		case.defi_user,
		case.merkle_proofs)
	verify {
		assert_last_event::<T>(Event::ClaimSuccess(icon_address.clone()).into());
	}

	dispatch_exchange_claim {
		let x in 0 .. 3;

		Pallet::<T>::set_creditor_account(creditor_key);
		let system_account_id = Pallet::<T>::get_creditor_account();
		Pallet::<T>::init_balance(&system_account_id,10_000_000_000);
		let case= UserClaimTestCase::<<T as pallet::Config>::MaxProofSize>::try_from(benchmark_samples[x as usize].clone()).unwrap();
		let amount = <T::BalanceTypeConversion as Convert<_, _>>::convert(case.amount);
		let icon_address=case.icon_address.clone();
		<ExchangeAccountsMap<T>>::insert(icon_address.clone(),amount);


	}: dispatch_exchange_claim(
		RawOrigin::Root,
		icon_address.clone(),
		case.ice_address,
		amount,
		case.defi_user,
		case.merkle_proofs)
	verify {
		assert_last_event::<T>(Event::ClaimSuccess(icon_address.clone()).into());
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
