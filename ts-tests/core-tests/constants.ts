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
	CHARLIE: "//Charlie",
	DAVE: "//Dave",
	EVE: "//Eve",
	FERDIE: "//Ferdie",
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
