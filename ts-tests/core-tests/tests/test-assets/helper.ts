import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import BigNumber from "bignumber.js";
import { SnowApi } from "../../services";

// get asset balance
export async function getAssetBalance(api: ApiPromise, assetId: Number, queryAddr: string) {
	const bal = await api.query.assets.account(assetId, queryAddr);
	// @ts-ignore
	return bal.toJSON() ? bal.toJSON().balance.toString() : "0";
}

// create asset
export async function createAsset(
	txOrigin: KeyringPair,
	api: ApiPromise,
	id: Number,
	adminAddr: string,
	minBalance: BigNumber,
) {
	return new Promise(async (resolve, reject) => {
		const unsub = await api.tx.assets
			.create(id, adminAddr, minBalance.toFixed(0))
			.signAndSend(txOrigin, {}, ({ status, dispatchError }) => {
				if (status.isInBlock || status.isFinalized) {
					try {
						SnowApi.checkError(dispatchError);
					} catch (err) {
						return reject(err);
					}
					unsub();
					return resolve(true);
				}
			});
	});
}

// mint asset
export async function mintAsset(
	txOrigin: KeyringPair,
	api: ApiPromise,
	id: Number,
	beneficiaryAddr: string,
	amount: BigNumber,
) {
	return new Promise(async (resolve, reject) => {
		const unsub = await api.tx.assets
			.mint(id, beneficiaryAddr, amount.toFixed(0))
			.signAndSend(txOrigin, {}, ({ status, dispatchError }) => {
				if (status.isInBlock || status.isFinalized) {
					try {
						SnowApi.checkError(dispatchError);
					} catch (err) {
						return reject(err);
					}
					unsub();
					return resolve(true);
				}
			});
	});
}

// transfer asset
export async function transferAsset(
	txOrigin: KeyringPair,
	api: ApiPromise,
	id: Number,
	beneficiaryAddr: string,
	amount: BigNumber,
) {
	return new Promise(async (resolve, reject) => {
		const unsub = await api.tx.assets
			.transfer(id, beneficiaryAddr, amount.toFixed(0))
			.signAndSend(txOrigin, {}, ({ status, dispatchError }) => {
				if (status.isInBlock || status.isFinalized) {
					try {
						SnowApi.checkError(dispatchError);
					} catch (err) {
						return reject(err);
					}
					unsub();
					return resolve(true);
				}
			});
	});
}

// balance less than min balance cannot be transferred
// account with balance falling below min balance should show 0 balance

// update asset

// destroy asset
