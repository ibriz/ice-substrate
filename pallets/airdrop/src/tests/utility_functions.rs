use super::prelude::*;

#[test]
fn update_offchain_account() {
	minimal_test_ext().execute_with(|| {
		assert_noop!(
			AirdropModule::set_offchain_account(Origin::none(), samples::ACCOUNT_ID[1]),
			PalletError::DeniedOperation
		);

		assert_noop!(
			AirdropModule::set_offchain_account(
				Origin::signed(samples::ACCOUNT_ID[1]),
				samples::ACCOUNT_ID[2]
			),
			PalletError::DeniedOperation
		);

		assert_ok!(AirdropModule::set_offchain_account(
			Origin::root(),
			samples::ACCOUNT_ID[1]
		));
		assert_eq!(
			Some(samples::ACCOUNT_ID[1]),
			AirdropModule::get_offchain_account()
		);
	});
}

#[test]
fn ensure_root_or_offchain() {
	minimal_test_ext().execute_with(|| {
		use sp_runtime::DispatchError::BadOrigin;

		// root origin should pass
		assert_ok!(AirdropModule::ensure_root_or_offchain(Origin::root()));

		// Any signed other than offchian account should fail
		assert_err!(
			AirdropModule::ensure_root_or_offchain(Origin::signed(not_offchain_account(
				samples::ACCOUNT_ID[1]
			))),
			BadOrigin
		);

		// Unsigned origin should fail
		assert_err!(
			AirdropModule::ensure_root_or_offchain(Origin::none()),
			BadOrigin
		);

		// Signed with offchain account should work
		assert_ok!(AirdropModule::set_offchain_account(
			Origin::root(),
			samples::ACCOUNT_ID[1]
		));
		assert_ok!(AirdropModule::ensure_root_or_offchain(Origin::signed(
			samples::ACCOUNT_ID[1]
		)));
	});
}



//#[test]
// fn pending_claims_getter() {
// 	type PendingClaimsOf = types::PendingClaimsOf<Test>;
// 	use samples::ICON_ADDRESS;

// 	let get_flattened_vec = |mut walker: PendingClaimsOf| {
// 		let mut res: Vec<(types::BlockNumberOf<Test>, types::IconAddress)> = vec![];

// 		while let Some((bl_num, mut inner_walker)) = walker.next() {
// 			while let Some(entry) = inner_walker.next() {
// 				res.push((bl_num, entry));
// 			}
// 		}

// 		res
// 	};

// 	let sample_entries: &[(types::BlockNumberOf<Test>, types::IconAddress)] = &[
// 		(1_u32.into(), ICON_ADDRESS[0]),
// 		(1_u32.into(), ICON_ADDRESS[1]),
// 		(2_u32.into(), ICON_ADDRESS[3]),
// 		(10_u32.into(), ICON_ADDRESS[2]),
// 	];

// 	const EMPTY: [(types::BlockNumberOf<Test>, types::IconAddress); 0] = [];

// 	minimal_test_ext().execute_with(|| {
// 		// When there is nothing in storage it should always return empty entry
// 		{
// 			let entries = get_flattened_vec(PendingClaimsOf::new(1_u32.into()..5_u32.into()));
// 			assert_eq!(EMPTY.to_vec(), entries);
// 		}

// 		// Make some data entry with dummy retry count
// 		for (k1, k2) in sample_entries {
// 			pallet_airdrop::PendingClaims::<Test>::insert(k1, k2, 1_u8);
// 		}

// 		// Make sure range is treated as exclusive
// 		{
// 			let entries = get_flattened_vec(PendingClaimsOf::new(0_u32.into()..1_u32.into()));
// 			assert_eq!(EMPTY.to_vec(), entries);

// 			let entries = get_flattened_vec(PendingClaimsOf::new(10_u32.into()..10_u32.into()));
// 			assert_eq!(EMPTY.to_vec(), entries);

// 			let entries = get_flattened_vec(PendingClaimsOf::new(10_u32.into()..20_u32.into()));
// 			assert_eq!(vec![(10_u32.into(), ICON_ADDRESS[2])], entries);
// 		}

// 		// Make sure out of range is always empty
// 		{
// 			let entries = get_flattened_vec(PendingClaimsOf::new(20_u32.into()..30_u32.into()));
// 			assert_eq!(EMPTY.to_vec(), entries);
// 		}

// 		// Make sure correct data is returned
// 		{
// 			let entries = get_flattened_vec(PendingClaimsOf::new(1_u32.into()..3_u32.into()));
// 			assert_eq!(
// 				vec![
// 					(1_u32.into(), ICON_ADDRESS[0]),
// 					(1_u32.into(), ICON_ADDRESS[1]),
// 					(2_u32.into(), ICON_ADDRESS[3])
// 				],
// 				entries
// 			);
// 		}
// 	})
// }

#[test]
fn get_vesting_amounts_splitted() {
	minimal_test_ext().execute_with(|| {
		use sp_runtime::ArithmeticError;

		assert_err!(
			AirdropModule::get_splitted_amounts(types::ServerBalance::max_value(), true),
			ArithmeticError::Overflow
		);
		assert_eq!(
			Ok((0_u32.into(), 0_u32.into())),
			AirdropModule::get_splitted_amounts(0_u32.into(), true)
		);

		assert_eq!(
			Ok((900_u32.into(), 2100_u32.into())),
			AirdropModule::get_splitted_amounts(3_000_u32.into(), false)
		);
		assert_eq!(
			Ok((1200_u32.into(), 1800_u32.into())),
			AirdropModule::get_splitted_amounts(3_000_u32.into(), true)
		);

		assert_eq!(
			Ok((0_u32.into(), 1_u32.into())),
			AirdropModule::get_splitted_amounts(1_u32.into(), false)
		);
		assert_eq!(
			Ok((0_u32.into(), 1_u32.into())),
			AirdropModule::get_splitted_amounts(1_u32.into(), true)
		);

		assert_eq!(
			Ok((2932538_u32.into(), 6842591_u32.into())),
			AirdropModule::get_splitted_amounts(9775129_u32.into(), false)
		);
		assert_eq!(
			Ok((3910051_u32.into(), 5865078_u32.into())),
			AirdropModule::get_splitted_amounts(9775129_u32.into(), true)
		);
	});
}

#[test]
fn cook_vesting_schedule() {
	type BlockToBalance = <Test as pallet_vesting::Config>::BlockNumberToBalance;
	minimal_test_ext().execute_with(|| {
		{
			let (schedule, remainder) =
				utils::new_vesting_with_deadline::<Test, 0u32>(10u32.into(), 10u32.into());

			let schedule = schedule.unwrap();
			assert_eq!(remainder, 0u32.into());

			assert_eq!(schedule.locked(), 10u32.into());
			assert_eq!(schedule.per_block(), 1u32.into());
			assert_eq!(
				schedule.ending_block_as_balance::<BlockToBalance>(),
				10u32.into()
			);
		}

		{
			let (schedule, remainder) =
				utils::new_vesting_with_deadline::<Test, 0u32>(5u32.into(), 10u32.into());

			assert_eq!(None, schedule);
			assert_eq!(remainder, 5u32.into());
		}

		{
			let (schedule, remainer) =
				utils::new_vesting_with_deadline::<Test, 5u32>(12u32.into(), 10u32.into());

			let primary = schedule.unwrap();
			assert_eq!(remainer, 2u32.into());

			assert_eq!(primary.locked(), 10u32.into());
			assert_eq!(primary.per_block(), 2u32.into());
			assert_eq!(
				primary.ending_block_as_balance::<BlockToBalance>(),
				10u32.into()
			);
		}

		{
			let (schedule, remainer) =
				utils::new_vesting_with_deadline::<Test, 0u32>(16u32.into(), 10u32.into());

			let schedule = schedule.unwrap();
			assert_eq!(remainer, 6u32.into());

			assert_eq!(schedule.locked(), 10u32.into());
			assert_eq!(schedule.per_block(), 1u32.into());
			assert_eq!(
				schedule.ending_block_as_balance::<BlockToBalance>(),
				10u32.into()
			);
		}

		{
			let (schedule, remainer) =
				utils::new_vesting_with_deadline::<Test, 0u32>(3336553u32.into(), 10_000u32.into());

			let schedule = schedule.unwrap();
			assert_eq!(remainer, 6553u32.into());

			assert_eq!(schedule.locked(), 3330000u32.into());
			assert_eq!(schedule.per_block(), 333u32.into());
			assert_eq!(
				schedule.ending_block_as_balance::<BlockToBalance>(),
				10_000u32.into()
			);
		}
	});
}

//#[test]
// fn making_vesting_transfer() {
// 	minimal_test_ext().execute_with(|| {
// 		run_to_block(3);

// 		let server_response = samples::SERVER_DATA[1];
// 		let icon_address =samples::ICON_ADDRESS[0];
// 		type Currency = <Test as pallet_airdrop::Config>::Currency;
// 		// Fund creditor
// 		credit_creditor(u32::MAX);
// 		credit_creditor(u32::MAX);

// 		{
// 			let claimer = samples::ACCOUNT_ID[1];
// 			let mut snapshot = types::SnapshotInfo::<Test> {
// 				done_instant: false,
// 				done_vesting: false,
// 				ice_address: claimer.clone(),
// 				..Default::default()
// 			};

// 			assert_ok!(AirdropModule::do_transfer(&server_response, &mut snapshot,&icon_address));

// 			// Ensure all amount is being transferred
// 			assert_eq!(9775129_u128, Currency::free_balance(&claimer));

// 			// Make sure user is getting atleast of instant amount
// 			// might get more due to vesting remainder
// 			assert!(Currency::usable_balance(&claimer) >= 2932538_u32.into());

// 			// Make sure flags are updated
// 			assert!(snapshot.done_vesting && snapshot.done_instant);
// 		}

// 		// When instant transfer is true but not vesting
// 		{
// 			let claimer = samples::ACCOUNT_ID[2];
// 			let mut snapshot = types::SnapshotInfo::<Test> {
// 				done_instant: true,
// 				done_vesting: false,
// 				ice_address: claimer.clone(),
// 				..Default::default()
// 			};

// 			assert_ok!(AirdropModule::do_transfer(&server_response, &mut snapshot,&icon_address));

// 			// Ensure amount only accounting to vesting is transfererd
// 			let expected_transfer = {
// 				let vesting_amount: types::VestingBalanceOf<Test> =
// 					AirdropModule::get_splitted_amounts(
// 						utils::get_response_sum(&server_response).unwrap(),
// 						server_response.defi_user,
// 					)
// 					.unwrap()
// 					.1;
// 				let schedule = utils::new_vesting_with_deadline::<Test, 100u32>(
// 					vesting_amount,
// 					10_000u32.into(),
// 				)
// 				.0
// 				.unwrap();

// 				schedule.locked()
// 			};

// 			assert_eq!(expected_transfer, Currency::free_balance(&claimer));

// 			// Ensure flag is updates
// 			assert!(snapshot.done_vesting && snapshot.done_instant);
// 		}

// 		// When vesting is true but not done_instant
// 		{
// 			let claimer = samples::ACCOUNT_ID[3];
// 			let mut snapshot = types::SnapshotInfo::<Test> {
// 				done_instant: false,
// 				done_vesting: true,
// 				ice_address: claimer.clone(),
// 				..Default::default()
// 			};

// 			assert_ok!(AirdropModule::do_transfer(&server_response, &mut snapshot,&icon_address));

// 			// Ensure amount only accounting to instant is transferred
// 			let expected_transfer = {
// 				let (instant_amount, vesting_amount) = AirdropModule::get_splitted_amounts(
// 					utils::get_response_sum(&server_response).unwrap(),
// 					server_response.defi_user,
// 				)
// 				.unwrap();
// 				let remainder = utils::new_vesting_with_deadline::<Test, 100u32>(
// 					vesting_amount,
// 					10_000u32.into(),
// 				)
// 				.1;

// 				instant_amount + remainder
// 			};
// 			assert_eq!(expected_transfer, Currency::free_balance(&claimer));

// 			// Ensure flag is updated
// 			assert!(snapshot.done_vesting && snapshot.done_instant);
// 		}

// 		// When both are false
// 		// NOTE: such snapshot will not be passed in actual from complete_transfer
// 		{
// 			let claimer = samples::ACCOUNT_ID[4];
// 			let mut snapshot = types::SnapshotInfo::<Test> {
// 				done_instant: true,
// 				done_vesting: true,
// 				ice_address: claimer.clone(),
// 				..Default::default()
// 			};

// 			assert_ok!(AirdropModule::do_transfer(&server_response, &mut snapshot,&icon_address));

// 			// Ensure amount only accounting to instant is transfererd
// 			assert_eq!(0_u128, Currency::free_balance(&claimer));

// 			// Ensure flag is updates
// 			assert!(snapshot.done_vesting && snapshot.done_instant);
// 		}
// 	});
// }
