use super::prelude::*;
use pallet_vesting::VestingInfo;
type Vesting = pallet_vesting::Pallet<<Test as pallet_airdrop::Config>::VestingModule>;
type Currency = <Test as pallet_airdrop::Config>::Currency;

/*
#[test]
fn verify_vesting_schedule() {
	minimal_test_ext().execute_with(|| {
		let vesting_schedule = AirdropModule::get_vesting_schedule(10_000_u32.into());

		// Ensure this schedule is valid
		assert!(vesting_schedule.is_valid());

		// Amount locked must be equal as passed while creating
		assert_eq!(10_000, vesting_schedule.locked());

		// Start vesting after this block height
		assert_eq!(10, vesting_schedule.starting_block());

		// Release this much of amount per block
		assert_eq!(100, vesting_schedule.per_block());

		// Locked balance before starting blokck should be untouched
		assert_eq!(10_000, vesting_schedule.locked_at::<types::BlockNumToBalance>(5_u64));

		// After certaing block locked amount must decrease lineraly
		assert_eq!(9500, vesting_schedule.locked_at::<types::BlockNumToBalance>(15_u64));

		// All balance will be released after this block height
		assert_eq!(100, vesting_schedule.ending_block_as_balance::<types::BlockNumToBalance>());
	});
}
*/

#[test]
fn test_vesting() {
	minimal_test_ext().execute_with(|| {
		run_to_block(10);
		credit_creditor(200_00_000__u32);

		run_to_block(9);

		let user = samples::ACCOUNT_ID[1];
		let total_balance: types::BalanceOf<Test> = 10_000_u32.into();
		let per_block: types::BalanceOf<Test> = 1000_u32.into();
		let vesting_schedule: pallet_vesting::VestingInfo<
			types::BalanceOf<Test>,
			types::BlockNumberOf<Test>,
		> = pallet_vesting::VestingInfo::new(total_balance, per_block, 9_u32.into());

		assert_ok!(Vesting::vested_transfer(
			Origin::signed(AirdropModule::get_creditor_account()),
			user.clone(),
			vesting_schedule.clone(),
		));

		// Vesting is transparent while quering free balance
		assert_eq!(
			<Test as pallet_airdrop::Config>::Currency::free_balance(&user),
			total_balance
		);

		run_to_block(40);

		assert_ok!(Vesting::vest(Origin::signed(user.clone())));

		// After passing starting block we must have some usable fund
		assert_ok!(AirdropModule::donate_to_creditor(
			Origin::signed(user.clone()),
			600_u32.into(),
			false
		));
	});
}

#[test]
fn can_user_cancel() {
	minimal_test_ext().execute_with(|| {
		let user = samples::ACCOUNT_ID[1];
		let total_balance: types::BalanceOf<Test> = 1000_u32.into();
		let per_block: types::BalanceOf<Test> = 100_u32.into();
		let vesting_schedule: pallet_vesting::VestingInfo<
			types::BalanceOf<Test>,
			types::BlockNumberOf<Test>,
		> = pallet_vesting::VestingInfo::new(total_balance, per_block, 10_u32.into());

		assert_ok!(pallet_vesting::Pallet::<
			<Test as pallet_airdrop::Config>::VestingModule,
		>::vested_transfer(
			Origin::signed(AirdropModule::get_creditor_account()),
			user.clone(),
			vesting_schedule.clone(),
		));

		run_to_block(11);

		// We are ok that vested fund will not be in use
		assert_err!(
			AirdropModule::donate_to_creditor(Origin::signed(user.clone()), 300_u32.into(), false),
			BalanceError::LiquidityRestrictions
		);

		// Let us suppose user called pallet_vesting::vest
		assert_ok!(pallet_vesting::Pallet::<
			<Test as pallet_airdrop::Config>::VestingModule,
		>::vest(Origin::signed(user.clone())));

		assert_err!(
			AirdropModule::donate_to_creditor(Origin::signed(user.clone()), 300_u32.into(), false),
			BalanceError::LiquidityRestrictions
		);
	});
}

fn credit_creditor(balance: u32) {
	let creditor_account = AirdropModule::get_creditor_account();
	let deposit_res = <Test as pallet_airdrop::Config>::Currency::set_balance(
		mock::Origin::root(),
		creditor_account,
		balance.into(),
		10_000_u32.into(),
	);

	assert_ok!(deposit_res);
}
