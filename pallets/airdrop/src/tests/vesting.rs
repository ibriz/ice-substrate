use super::prelude::*;
use pallet_vesting::VestingInfo;

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
		credit_creditor(200_000_u32);

		let user = samples::ACCOUNT_ID[1];
		let total_balance: types::BalanceOf<Test> = 1000_u32.into();
		let per_block: types::BalanceOf<Test> = 10_u32.into();
		let vesting_schedule: pallet_vesting::VestingInfo<
			types::BalanceOf<Test>,
			types::BlockNumberOf<Test>,
		> = pallet_vesting::VestingInfo::new(total_balance, per_block, 11_u32.into());

		assert_ok!(pallet_vesting::Pallet::<
			<Test as pallet_airdrop::Config>::VestingModule,
		>::vested_transfer(
			Origin::signed(AirdropModule::get_creditor_account()),
			user.clone(),
			vesting_schedule.clone(),
		));

		run_to_block(11);
		assert_eq!(10_u128, get_free_balance(&user));

		run_to_block(50);
		assert_eq!(100_u128, get_free_balance(&user));
	});
}

fn get_free_balance(account: &types::AccountIdOf<Test>) -> types::BalanceOf<Test> {
	<Test as pallet_airdrop::Config>::Currency::free_balance(account)
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
	assert_eq!(get_free_balance(&creditor_account), balance.into());
}
