use super::prelude::*;

fn get_free_balance(account: &types::AccountIdOf<Test>) -> types::BalanceOf<Test> {
	<Test as pallet_airdrop::Config>::Currency::free_balance(account)
}
