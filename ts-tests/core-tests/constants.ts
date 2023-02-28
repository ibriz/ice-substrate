export const BINARY_PATH = `../../target/release/ice-node`;

export const LOCAL_WSS_URL = "ws://localhost:9944";
export const LOCAL_CHAIN_PREFIX = 2208;

export const KEYRING_TYPE = "sr25519";

export const BLOCK_TIME_MS = 12_000;

export const WALLET_SEED = {
	ALICE: "//Alice",
	ALICE_STASH: "//Alice//stash",
	BOB: "//Bob",
	BOB_STASH: "//Bob//stash",
	WALLET_1: "frown toast shoe aunt slender chicken dirt oyster royal wish must mistake",
	WALLET_2: "prison athlete claw suggest van come buzz avoid kangaroo scrub east elegant",
	WALLET_3: "organ sudden guitar prepare sword tuition prosper barrel grant misery ladder simple",
};

export const CHAINS = {
	snow: {
		RPC_ENDPOINT: "wss://snow-rpc.icenetwork.io",
		CHAIN_ID: 552,
		CHAIN_PREFIX: 2207,
	},
	arctic: {
		RPC_ENDPOINT: "wss://arctic-rpc.icenetwork.io:9944",
		CHAIN_ID: 553,
		CHAIN_PREFIX: 2208,
	},
	snow_staging: {
		RPC_ENDPOINT: "wss://snow-staging-rpc.web3labs.com:9944",
		CHAIN_ID: 552,
		CHAIN_PREFIX: 2207,
	},
	local: {
		RPC_ENDPOINT: "ws://localhost:9944",
		CHAIN_ID: 554,
		CHAIN_PREFIX: 2208,
	},
};
