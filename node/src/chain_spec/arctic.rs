use super::{get_from_seed, Extensions};
use arctic_runtime::currency::ICY;
use arctic_runtime::{
	wasm_binary_unwrap, AccountId, AuraConfig, AuraId,AirdropConfig, Balance, BalancesConfig,
	CollatorSelectionConfig, CouncilConfig, EVMConfig, GenesisConfig, ParachainInfoConfig,
	SessionConfig, SessionKeys, Signature, SudoConfig, SystemConfig, VestingConfig,
};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::collections::BTreeMap;
use std::marker::PhantomData;

/// Publicly expose ArcticChainSpec for sc service
pub type ArcticChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

const AIRDROP_MERKLE_ROOT:[u8; 32] =hex_literal::hex!("990e01e3959627d2ddd94927e1c605a422b62dc3b8c8b98d713ae6833c3ef122");

const AIRDROP_EXCHANGE_ACCOUNTS: &[([u8; 20], u128)] = &[
	(
		hex_literal::hex!("562dc1e2c7897432c298115bc7fbcc3b9d5df294"),
		70717613544517522852341727,
	),
	(
		hex_literal::hex!("61acc986a761b5f354dc8777360aeaf47b2ab616"),
		8968750000000000000,
	),
	(
		hex_literal::hex!("6d14b2b77a9e73c5d5804d43c7e3c3416648ae3d"),
		8348890436199324029817984,
	),
	(
		hex_literal::hex!("938b9a413de9ffbbeae72e7034931a3bdf0f1e96"),
		2959971579000000000000000 + 13972742011003245391080,
	),
	(
		hex_literal::hex!("d182113fea7ae3164871bfda90ec8652123aa354"),
		352948797792142357220773,
	),
];

const AIRDROP_CREDITOR_ACCOUNT:[u8; 32] = hex_literal::hex!("10b3ae7ebb7d722c8e8d0d6bf421f6d5dbde8d329f7c905a201539c635d61872");


const ARCTIC_PROPERTIES: &str = r#"
        {
            "ss58Format": 42,
            "tokenDecimals": 18,
            "tokenSymbol": "ICZ"
        }"#;

/// Gen Arctic chain specification for given parachain id.
pub fn get_chain_spec(para_id: u32) -> ArcticChainSpec {
	ArcticChainSpec::from_genesis(
		"Arctic Testnet",
		"arctic",
		ChainType::Live,
		move || {
			make_genesis(
				// Endowed accounts
				vec![
					hex!["62687296bffd79f12178c4278b9439d5eeb8ed7cc0b1f2ae29307e806a019659"].into(),
					hex!["d893ef775b5689473b2e9fa32c1f15c72a7c4c86f05f03ee32b8aca6ce61b92c"].into(),
					hex!["98003761bff94c8c44af38b8a92c1d5992d061d41f700c76255c810d447d613f"].into(),
				],
				// Initial PoA authorities
				vec![
					(
						hex!["62687296bffd79f12178c4278b9439d5eeb8ed7cc0b1f2ae29307e806a019659"]
							.into(),
						hex!["62687296bffd79f12178c4278b9439d5eeb8ed7cc0b1f2ae29307e806a019659"]
							.unchecked_into(),
					),
					(
						hex!["d893ef775b5689473b2e9fa32c1f15c72a7c4c86f05f03ee32b8aca6ce61b92c"]
							.into(),
						hex!["d893ef775b5689473b2e9fa32c1f15c72a7c4c86f05f03ee32b8aca6ce61b92c"]
							.unchecked_into(),
					),
				],
				// Council members
				vec![
					hex!["62687296bffd79f12178c4278b9439d5eeb8ed7cc0b1f2ae29307e806a019659"].into(),
				],
				// Sudo account
				hex!["62687296bffd79f12178c4278b9439d5eeb8ed7cc0b1f2ae29307e806a019659"].into(),
				para_id.into(),
				// Creditor account
				AIRDROP_CREDITOR_ACCOUNT.clone().into(),
				// Airdrop merkle root
				AIRDROP_MERKLE_ROOT.clone(),
				// Airdrop exchange account
				AIRDROP_EXCHANGE_ACCOUNTS.to_vec(),
			)
		},
		vec![],
		None,
		None,
		None,
		serde_json::from_str(ARCTIC_PROPERTIES).unwrap(),
		Extensions {
			bad_blocks: Default::default(),
			relay_chain: "arctic".into(),
			para_id,
		},
	)
}

/// Gen Arctic chain specification for given parachain id.
pub fn get_dev_chain_spec(para_id: u32) -> ArcticChainSpec {
	// Alice as default
	let sudo_key = get_account_id_from_seed::<sr25519::Public>("Alice");
	let endowned = vec![
		(get_account_id_from_seed::<sr25519::Public>("Alice")),
		(get_account_id_from_seed::<sr25519::Public>("Bob")),
	];

	ArcticChainSpec::from_genesis(
		"Arctic Dev",
		"arctic-dev",
		ChainType::Development,
		move || {
			make_genesis(
				// Endowed accounts
				endowned.clone(),
				// Initial PoA authorities
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_from_seed::<AuraId>("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_from_seed::<AuraId>("Bob"),
					),
				],
				// Council members
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
				// Sudo account
				sudo_key.clone(),
				// Parachain id
				para_id.into(),
				// Creditor account
				AIRDROP_CREDITOR_ACCOUNT.clone().into(),
				// Airdrop merkle root
				AIRDROP_MERKLE_ROOT.clone(),
				// Airdrop exchange account
				AIRDROP_EXCHANGE_ACCOUNTS.to_vec(),
			)
		},
		vec![],
		None,
		None,
		None,
		serde_json::from_str(ARCTIC_PROPERTIES).unwrap(),
		Extensions {
			bad_blocks: Default::default(),
			relay_chain: "arctic".into(),
			para_id,
		},
	)
}

/// Helper for session keys to map aura id
fn session_keys(aura: AuraId) -> SessionKeys {
	SessionKeys { aura }
}

/// Helper function to create Arctic GenesisConfig.
fn make_genesis(
	endowed_accounts: Vec<AccountId>,
	authorities: Vec<(AccountId, AuraId)>,
	council_members: Vec<AccountId>,
	root_key: AccountId,
	parachain_id: ParaId,
	creditor_account: AccountId,
	airdrop_merkle_root: [u8; 32],
	airdrop_exchange_accounts: Vec<([u8; 20], u128)>,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
		},
		sudo: SudoConfig {
			key: Some(root_key),
		},
		parachain_info: ParachainInfoConfig { parachain_id },
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, ICY * 300_000_000))
				.collect(),
		},
		vesting: VestingConfig { vesting: vec![] },
		aura: AuraConfig {
			authorities: vec![],
		},
		aura_ext: Default::default(),
		collator_selection: CollatorSelectionConfig {
			desired_candidates: 200,
			candidacy_bond: 32_000 * ICY,
			invulnerables: authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		},
		session: SessionConfig {
			keys: authorities
				.iter()
				.map(|x| (x.0.clone(), x.0.clone(), session_keys(x.1.clone())))
				.collect::<Vec<_>>(),
		},
		evm: EVMConfig {
			accounts: {
				let map = BTreeMap::new();
				map
			},
		},
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
		assets: Default::default(),
		council: CouncilConfig {
			members: council_members,
			phantom: PhantomData,
		},
		ethereum: Default::default(),
		treasury: Default::default(),
		polkadot_xcm: Default::default(),
		parachain_system: Default::default(),
		airdrop: AirdropConfig {
			creditor_account: creditor_account,
			merkle_root: airdrop_merkle_root,
			exchange_accounts: airdrop_exchange_accounts,
		},
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}
