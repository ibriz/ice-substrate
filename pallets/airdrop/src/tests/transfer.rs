use super::prelude::*;

#[test]
fn donate_to_creditor() {
	minimal_test_ext().execute_with(|| {
		let donator = samples::ACCOUNT_ID[1];
		let creditor_account = AirdropModule::get_creditor_account();
		let donating_amount: types::BalanceOf<Test> = 1_00_000_u32.into();

		// Make sure donation will succeed as expected
		{
			// First create a user with some prefunded balance
			assert_ok!(pallet_balances::Pallet::<Test>::set_balance(
				Origin::root(),
				donator.clone(),
				donating_amount * 2_128,
				10_000_u32.into(),
			));

			// Record creditor balance before receiving donation
			let pre_donation_balance = get_free_balance(&creditor_account);

			// Do the donation and esnure it is ok
			assert_ok!(AirdropModule::donate_to_creditor(
				Origin::signed(donator.clone()),
				donating_amount,
				false
			));

			// Record creditor balance after receiving donaiton
			let post_donation_balance = get_free_balance(&creditor_account);

			// Make sure creditor is credited to exact donating amount
			assert_eq!(
				donating_amount,
				post_donation_balance - pre_donation_balance
			);
		}
	});
}

fn get_free_balance(account: &types::AccountIdOf<Test>) -> types::BalanceOf<Test> {
	<Test as pallet_airdrop::Config>::Currency::free_balance(account)
}
