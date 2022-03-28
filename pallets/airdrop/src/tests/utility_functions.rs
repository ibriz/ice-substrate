use super::prelude::*;

#[test]
fn pool_dispatchable_from_offchain() {
	let (mut test_ext, _, pool_state, _) = offchain_test_ext();

	test_ext.execute_with(|| {
		let calls = [
			&PalletCall::claim_request {
				icon_address: samples::ICON_ADDRESS[0],
				message: b"icx_sendTransaction.data.{method.transfer.params.{wallet.da8db20713c087e12abae13f522693299b9de1b70ff0464caa5d392396a8f76c}}.dataType.call.from.hxdd9ecb7d3e441d25e8c4f03cd20a80c502f0c374.nid.0x1.nonce.0x1..timestamp.0x5d56f3231f818.to.cx8f87a4ce573a2e1377545feabac48a960e8092bb.version.0x3".to_vec(),
				icon_signature: bytes::from_hex("0xa64874af3653").unwrap(),
			},
			&PalletCall::donate_to_creditor {
				amount: 10_00_u32.into(),
				allow_death: true,
			},
			&PalletCall::register_failed_claim {
				block_number: 1_u32.into(),
				icon_address: types::IconAddress::default(),
			},
		];

		assert_ok!(AirdropModule::make_signed_call(&calls[0]));
		assert_tx_call(&calls[..1], &pool_state.read());
		
		assert_ok!(AirdropModule::make_signed_call(&calls[1]));
		assert_tx_call(&calls[..2], &pool_state.read());
		
		assert_ok!(AirdropModule::make_signed_call(&calls[2]));
		assert_tx_call(&calls[..3], &pool_state.read());
	});
}

#[test]
fn update_offchain_account() {
	minimal_test_ext().execute_with(||{
		assert_noop!(
			AirdropModule::set_offchain_account(Origin::none(), samples::ACCOUNT_ID[1]),
			PalletError::DeniedOperation
		);

		assert_noop!(
			AirdropModule::set_offchain_account(Origin::signed(samples::ACCOUNT_ID[1]), samples::ACCOUNT_ID[2]),
			PalletError::DeniedOperation
		);

		assert_ok!(AirdropModule::set_offchain_account(Origin::root(), samples::ACCOUNT_ID[1]));
		assert_eq!(Some(samples::ACCOUNT_ID[1]), AirdropModule::get_offchain_account());
	});
}

#[test]
fn ensure_root_or_offchain() {
	minimal_test_ext().execute_with(|| {
		use sp_runtime::DispatchError::BadOrigin;

		// root origin should pass
		assert_ok!(AirdropModule::ensure_root_or_offchain(Origin::root()));

		// Any signed other than offchian account should fail
		assert_err!(AirdropModule::ensure_root_or_offchain(Origin::signed(not_offchain_account(samples::ACCOUNT_ID[1]))), BadOrigin);

		// Unsigned origin should fail
		assert_err!(AirdropModule::ensure_root_or_offchain(Origin::none()), BadOrigin);

		// Signed with offchain account should work
		assert_ok!(AirdropModule::set_offchain_account(Origin::root(), samples::ACCOUNT_ID[1]));
		assert_ok!(AirdropModule::ensure_root_or_offchain(Origin::signed(samples::ACCOUNT_ID[1])));

	});
}

#[test]
fn making_correct_http_request() {
	let icon_address = samples::ICON_ADDRESS[0];

	let (mut test_ext, offchain_state,_,_) = offchain_test_ext();
	put_response(
		&mut offchain_state.write(),
		&icon_address,
		&serde_json::to_string(&samples::SERVER_DATA[0]).unwrap(),
	);

	test_ext.execute_with(|| {
		let fetch_res = AirdropModule::fetch_from_server(&icon_address);
		assert_ok!(fetch_res);
	});
}

#[test]
fn failed_entry_regestration() {
	minimal_test_ext().execute_with(|| {
		let bl_num: types::BlockNumberOf<Test> = 2_u32.into();
		let claimer = samples::ICON_ADDRESS[0];
		let retry = 2_u8;
		let running_bl_num = bl_num + 6;

		// Simulate we running in block running_bl_num;
		mock::System::set_block_number(running_bl_num);

		// Be sure access is controlled
		{
			assert_storage_noop!(assert_eq! {
				AirdropModule::register_failed_claim(
					Origin::signed(not_offchain_account(samples::ACCOUNT_ID[1])),
					bl_num.into(),
					claimer.clone(),
				)
				.unwrap_err(),

				PalletError::DeniedOperation.into()
			});

			assert_storage_noop!(assert_eq! {
				AirdropModule::register_failed_claim(
					Origin::none(),
					bl_num.into(),
					claimer.clone(),
				)
				.unwrap_err(),

				PalletError::DeniedOperation.into()
			});

			assert_ok!(AirdropModule::set_offchain_account(Origin::root(), samples::ACCOUNT_ID[2]));
			assert_storage_noop!(assert_ne! {
				AirdropModule::register_failed_claim(
					Origin::signed(AirdropModule::get_offchain_account().unwrap()),
					bl_num.into(),
					claimer.clone(),
				)
				.unwrap_err(),

				PalletError::DeniedOperation.into()
			});
		}

		// When there is no data in map
		{
			assert_noop!(
				AirdropModule::register_failed_claim(Origin::root(), bl_num, claimer.clone()),
				PalletError::IncompleteData
			);
		}

		// Insert sample data in map
		pallet_airdrop::IceSnapshotMap::insert(&claimer, types::SnapshotInfo::<Test>::default());

		// When there is something in map but not in queue
		{
			assert_noop!(
				AirdropModule::register_failed_claim(Origin::root(), bl_num, claimer.clone()),
				PalletError::NotInQueue
			);
		}

		// Insert a sample data in queue with 0 retry remaining
		pallet_airdrop::PendingClaims::<Test>::insert(bl_num, &claimer, 0_u8);

		// When there are no more retry left in this entry
		{
			assert_ok!(
				AirdropModule::register_failed_claim(Origin::root(), bl_num, claimer.clone())
			);
			// Still entry should be removed from queue
			assert_eq!(None, AirdropModule::get_pending_claims(bl_num, &claimer));
		}

		// Reinsert in queue with some retry count left
		pallet_airdrop::PendingClaims::<Test>::insert(bl_num, &claimer, retry);

		// This should now succeed
		{
			assert_ok!(AirdropModule::register_failed_claim(
				Origin::root(),
				bl_num,
				claimer.clone()
			));

			// Make sure entry is no longer in old key
			assert_eq!(None, AirdropModule::get_pending_claims(bl_num, &claimer));

			// Make sure entry is shifter to another key with retry decremented
			assert_eq!(
				Some(retry - 1),
				AirdropModule::get_pending_claims(running_bl_num + 1, &claimer)
			);
		}
	});
}

#[test]
fn pending_claims_getter() {
	type PendingClaimsOf = types::PendingClaimsOf<Test>;
	use samples::ICON_ADDRESS;

	let get_flattened_vec = |mut walker: PendingClaimsOf| {
		let mut res: Vec<(types::BlockNumberOf<Test>, types::IconAddress)> = vec![];

		while let Some((bl_num, mut inner_walker)) = walker.next() {
			while let Some(entry) = inner_walker.next() {
				res.push((bl_num, entry));
			}
		}

		res
	};

	let sample_entries: &[(types::BlockNumberOf<Test>, types::IconAddress)] = &[
		(1_u32.into(),ICON_ADDRESS[0]),
		(1_u32.into(), ICON_ADDRESS[1]),
		(2_u32.into(), ICON_ADDRESS[3]),
		(10_u32.into(), ICON_ADDRESS[2]),
	];

	const EMPTY: [(types::BlockNumberOf<Test>, types::IconAddress); 0] = [];

	minimal_test_ext().execute_with(|| {
		// When there is nothing in storage it should always return empty entry
		{
			let entries = get_flattened_vec(PendingClaimsOf::new(1_u32.into()..5_u32.into()));
			assert_eq!(EMPTY.to_vec(), entries);
		}

		// Make some data entry with dummy retry count
		for (k1, k2) in sample_entries {
			pallet_airdrop::PendingClaims::<Test>::insert(k1, k2, 1_u8);
		}

		// Make sure range is treated as exclusive
		{
			let entries = get_flattened_vec(PendingClaimsOf::new(0_u32.into()..1_u32.into()));
			assert_eq!(EMPTY.to_vec(), entries);

			let entries = get_flattened_vec(PendingClaimsOf::new(10_u32.into()..10_u32.into()));
			assert_eq!(EMPTY.to_vec(), entries);

			let entries = get_flattened_vec(PendingClaimsOf::new(10_u32.into()..20_u32.into()));
			assert_eq!(
				vec![(10_u32.into(),ICON_ADDRESS[2])],
				entries
			);
		}

		// Make sure out of range is always empty
		{
			let entries = get_flattened_vec(PendingClaimsOf::new(20_u32.into()..30_u32.into()));
			assert_eq!(EMPTY.to_vec(), entries);
		}

		// Make sure correct data is returned
		{
			let entries = get_flattened_vec(PendingClaimsOf::new(1_u32.into()..3_u32.into()));
			assert_eq!(
				vec![
					(1_u32.into(), ICON_ADDRESS[0]),
					(1_u32.into(), ICON_ADDRESS[1]),
					(2_u32.into(), ICON_ADDRESS[3])
				],
				entries
			);
		}
	})
}

#[test]
fn get_vesting_amounts_splitted() {
	minimal_test_ext().execute_with(||{
		use sp_runtime::ArithmeticError;

		assert_err!(AirdropModule::get_vesting_amounts(u128::MAX, true), ArithmeticError::Overflow);
		assert_eq!(Ok([0_u128, 0_u128]), AirdropModule::get_vesting_amounts(0_u128, true));

		assert_eq!(Ok([900_u128, 2100_u128]), AirdropModule::get_vesting_amounts(3_000_u128, false));
		assert_eq!(Ok([1200_u128, 1800_u128]), AirdropModule::get_vesting_amounts(3_000_u128, true));
		
		assert_eq!(Ok([0_u128, 1_u128]), AirdropModule::get_vesting_amounts(1_u128, false));
		assert_eq!(Ok([0_u128, 1_u128]), AirdropModule::get_vesting_amounts(1_u128, true));

		assert_eq!(Ok([2932538_u128, 6842591_u128]), AirdropModule::get_vesting_amounts(9775129_u128, false));
		assert_eq!(Ok([3910051_u128, 5865078_u128]), AirdropModule::get_vesting_amounts(9775129_u128, true));
	});
}

#[test]
fn cooking_vesting_schedule() {
	minimal_test_ext().execute_with(||{
		run_to_block(10);

		let server_response = samples::SERVER_DATA[1];
		
		let vesting_res = AirdropModule::make_vesting_schedule(&server_response);
		assert_ok!(vesting_res);
		let [first_vesting, second_vesting] = vesting_res.unwrap();

		let first_vest_amount = 2932538_u128;
		let expected_first_vesting = types::VestingInfoOf::<Test>::new(first_vest_amount, first_vest_amount, 9_u64);
		assert_eq!(expected_first_vesting, first_vesting);

		let second_vest_amount = 6842591_u128;
		let expected_second_vesting = types::VestingInfoOf::<Test>::new(second_vest_amount, 4, 11_u64);
		assert_eq!(expected_second_vesting, second_vesting);
	});
}

#[test]
fn making_vesting_transfer() {
	minimal_test_ext().execute_with(||{
		run_to_block(3);

		let server_response = samples::SERVER_DATA[1];
		let claimer = samples::ACCOUNT_ID[1];
		type Currency = <Test as pallet_airdrop::Config>::Currency;

		// Fund creditor
		credit_creditor(u32::MAX);

		assert_ok!(AirdropModule::do_vested_transfer(claimer, &server_response));

		// Ensure all amount is being transferred
		assert_eq!(9775129_u128, Currency::free_balance(&claimer));

		// First vesting should be usable instantly
		assert_eq!(2932538_u128, Currency::usable_balance(&claimer));
	});
}