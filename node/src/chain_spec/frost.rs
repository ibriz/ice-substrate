use frost_runtime::{
	currency::ICY, opaque::SessionKeys, AccountId, AirdropConfig, AuraConfig, BalancesConfig,
	CouncilConfig, EVMConfig, EthereumConfig, GenesisConfig, GrandpaConfig, PalletId,
	SessionConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use hex_literal::hex;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{AccountIdConversion, IdentifyAccount, Verify};
use std::collections::BTreeMap;
use std::marker::PhantomData;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const FROST_PROPERTIES: &str = r#"
        {
            "ss58Format": 42,
            "tokenDecimals": 18,
            "tokenSymbol": "ICZ"
        }"#;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type FrostChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

// Treasury Pallet ID
const TREASURY_PALLET_ID: PalletId = PalletId(*b"py/trsry");

const AIRDROP_MERKLE_ROOT: [u8; 32] =
	hex_literal::hex!("990e01e3959627d2ddd94927e1c605a422b62dc3b8c8b98d713ae6833c3ef122");

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

const AIRDROP_CREDITOR_ACCOUNT: [u8; 32] =
	hex_literal::hex!("10b3ae7ebb7d722c8e8d0d6bf421f6d5dbde8d329f7c905a201539c635d61872");

/// Initialize frost testnet configuration
pub fn testnet_config() -> Result<FrostChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(FrostChainSpec::from_genesis(
		// Name
		"Frost Testnet",
		// ID
		"frost_testnet",
		ChainType::Custom(String::from("frost")),
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					(
						// AuraId
						hex!["62687296bffd79f12178c4278b9439d5eeb8ed7cc0b1f2ae29307e806a019659"]
							.unchecked_into(),
						// GrandpaId
						hex!["27c6da25d03bb6b3c751da3e8c5265b0bb357c15240602443cc286c0658b47f9"]
							.unchecked_into(),
					),
					(
						hex!["d893ef775b5689473b2e9fa32c1f15c72a7c4c86f05f03ee32b8aca6ce61b92c"]
							.unchecked_into(),
						hex!["85ec524aeacb6e558619a10da82cdf787026209211d1b7462cb176d58f2add86"]
							.unchecked_into(),
					),
				],
				// Council members
				vec![
					hex!["62687296bffd79f12178c4278b9439d5eeb8ed7cc0b1f2ae29307e806a019659"].into(),
				],
				// Sudo account
				hex!["62687296bffd79f12178c4278b9439d5eeb8ed7cc0b1f2ae29307e806a019659"].into(),
				// Creditor account
				AIRDROP_CREDITOR_ACCOUNT.clone().into(),
				// Airdrop merkle root
				AIRDROP_MERKLE_ROOT.clone(),
				// Airdrop exchange account
				AIRDROP_EXCHANGE_ACCOUNTS.to_vec(),
				// Pre-funded accounts
				vec![
					TREASURY_PALLET_ID.into_account(),
					hex!["62687296bffd79f12178c4278b9439d5eeb8ed7cc0b1f2ae29307e806a019659"].into(),
					hex!["d893ef775b5689473b2e9fa32c1f15c72a7c4c86f05f03ee32b8aca6ce61b92c"].into(),
					hex!["98003761bff94c8c44af38b8a92c1d5992d061d41f700c76255c810d447d613f"].into(),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		serde_json::from_str(FROST_PROPERTIES).unwrap(),
		// Extensions
		None,
	))
}

/// Initialize frost development configuration
pub fn development_config() -> Result<FrostChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	Ok(FrostChainSpec::from_genesis(
		// Name
		"Frost Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Council members
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Creditor account
				AIRDROP_CREDITOR_ACCOUNT.clone().into(),
				// Airdrop merkle root
				AIRDROP_MERKLE_ROOT.clone(),
				// Airdrop exchange accounts pair
				AIRDROP_EXCHANGE_ACCOUNTS.to_vec(),
				// Pre-funded accounts
				vec![
					TREASURY_PALLET_ID.into_account(),
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		serde_json::from_str(FROST_PROPERTIES).unwrap(),
		// Extensions
		None,
	))
}

/// Initialize frost local testnet configuration
pub fn local_testnet_config() -> Result<FrostChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(FrostChainSpec::from_genesis(
		// Name
		"Frost Local Testnet",
		// ID
		"frost_local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					authority_keys_from_seed("Alice"),
					authority_keys_from_seed("Bob"),
				],
				// Council members
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Creditor account,
				AIRDROP_CREDITOR_ACCOUNT.clone().into(),
				// Airdrop merkle root
				AIRDROP_MERKLE_ROOT.clone(),
				// Exhcnage accounts for Airdrop
				AIRDROP_EXCHANGE_ACCOUNTS.to_vec(),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
		None,
	))
}

/// Helper for session keys to map aura id
fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
	SessionKeys { aura, grandpa }
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	council_members: Vec<AccountId>,
	root_key: AccountId,
	creditor_account: AccountId,
	airdrop_merkle_root: [u8; 32],
	airdrop_exchange_accounts: Vec<([u8; 20], u128)>,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	let authorities = vec![
		(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			authority_keys_from_seed("Alice").0,
			authority_keys_from_seed("Alice").1,
		),
		(
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			authority_keys_from_seed("Bob").0,
			authority_keys_from_seed("Bob").1,
		),
	];
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, ICY * 40_000))
				.collect(),
		},
		aura: AuraConfig {
			authorities: vec![],
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		session: SessionConfig {
			keys: authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.1.clone(), x.2.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		evm: EVMConfig {
			accounts: {
				let map = BTreeMap::new();
				map
			},
		},
		ethereum: EthereumConfig {},
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
		vesting: Default::default(),
		assets: Default::default(),
		council: CouncilConfig {
			members: council_members,
			phantom: PhantomData,
		},
		treasury: Default::default(),
		airdrop: AirdropConfig {
			creditor_account: creditor_account,
			merkle_root: airdrop_merkle_root,
			exchange_accounts: airdrop_exchange_accounts,
		},
	}
}
