import { ChildProcessWithoutNullStreams, spawn } from "child_process";
import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { DispatchError } from "@polkadot/types/interfaces";
import BigNumber from "bignumber.js";
import { sleep } from "./helpers";
import { BINARY_PATH, KEYRING_TYPE, LOCAL_CHAIN_PREFIX, CHAINS, LOCAL_WSS_URL, WALLET_SEED } from "../../constants";
import { BlockInterface } from "../../interfaces/core";

const BUFFER_TIME = 12; // sec

class SnowApi {
	private static binary: undefined | ChildProcessWithoutNullStreams;
	static api: undefined | ApiPromise;
	public static keyring: undefined | Keyring;
	static wallets: Record<keyof typeof WALLET_SEED, KeyringPair> | undefined;

	static initialize = async (chain?: keyof typeof CHAINS) => {
		const RPC_ENDPOINT = chain ? CHAINS[chain].RPC_ENDPOINT : LOCAL_WSS_URL;
		if (!chain) {
			await SnowApi.startNetwork();
		}

		SnowApi.api = await SnowApi.connectSnowApi(RPC_ENDPOINT);
		SnowApi.keyring = new Keyring({
			type: KEYRING_TYPE,
			ss58Format: chain ? CHAINS[chain].CHAIN_PREFIX : LOCAL_CHAIN_PREFIX,
		});
		SnowApi.wallets = SnowApi.getWalletHandles();
	};

	private static getWalletHandles() {
		// @ts-ignore
		let wallets: typeof this.wallets = {};
		for (const [key, val] of Object.entries(WALLET_SEED)) {
			// @ts-ignore
			wallets[key] = SnowApi.keyring!.addFromUri(val);
		}
		return wallets;
	}

	static getNonce = async (address: string): Promise<number> => {
		// @ts-ignore
		const { nonce } = await SnowApi.api!.query.system.account(address);
		return parseInt(nonce.toHex());
	};

	private static connectSnowApi = async (nodeUrl: string) => {
		const provider = new WsProvider(nodeUrl);
		const _api = new ApiPromise({ provider });
		await _api.isReady;
		console.log(`Connected to ${nodeUrl}`);
		return _api;
	};

	private static startNetwork = async () => {
		const cmd = BINARY_PATH;
		const args = [`--dev`];
		SnowApi.binary = spawn(cmd, args);
		SnowApi.binary.on("error", (err) => {
			console.error(err);
			process.exit(-1);
		});
		// allow chain to be ready for ws connections
		await sleep(BUFFER_TIME);
	};

	static checkError = (errorObj: DispatchError | undefined) => {
		// Check if error occurred. If yes, set errorMsg.
		if (errorObj) {
			if (errorObj.isModule) {
				const decoded = SnowApi.api!.registry.findMetaError(errorObj.asModule);
				const { docs, name, section } = decoded;

				throw new Error(`${section}.${name}: ${docs}`);
			}
			else if (errorObj.isToken) {
				throw new Error(errorObj.asToken.toString());
			}
		}
	};

	static getLastBlock = async (): Promise<BlockInterface> => {
		const blockNum = (await this.api?.query.system.number())?.toString();
		const blockHash = (await this.api?.query.system.blockHash(blockNum))?.toString();
		if (blockNum && blockHash) {
			return { blockHash, blockNumber: parseInt(blockNum) };
		} else {
			throw new Error("Error fetching last block metadata");
		}
	};

	static getBlockHashByNumber = async (blockNum: number): Promise<string> => {
		const blockHash = await (await this.api?.query.system.blockHash(blockNum))?.toString();
		if (!blockHash) throw new Error(`Error getting block hash at block ${blockNum}`);
		return blockHash;
	};

	static getBalance = async (address: string, reserve?: boolean): Promise<BigNumber> => {
		const bal = await SnowApi.api?.query.system.account(address);
		// @ts-ignore
		return reserve ? new BigNumber(bal?.data.reserved.toBigInt()) : new BigNumber(bal?.data.free.toBigInt());
	};

	static sendBalance = (
		sender: KeyringPair,
		receiverAddr: string,
		weiAmount: BigNumber,
		options?: { nonce: number },
	) => {
		return new Promise(async (resolve, reject) => {
			const unsub = await SnowApi.api!.tx.balances.transfer(receiverAddr, weiAmount.toFixed(0)).signAndSend(
				sender,
				options ?? {},
				({ status, dispatchError }) => {
					if (status.isInBlock || status.isFinalized) {
						try {
							SnowApi.checkError(dispatchError);
						} catch (err) {
							return reject(err);
						}
						unsub();
						return resolve(true);
					}
				},
			);
		});
	};

	// static setBalance = () => {};

	static cleanUp = () => {
		this.api?.disconnect();
		this.binary?.kill();
	};
}

export default SnowApi;
