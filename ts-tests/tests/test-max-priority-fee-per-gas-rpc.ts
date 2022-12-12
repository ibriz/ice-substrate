import { ethers } from "ethers";
import { expect } from "chai";
import { step } from "mocha-steps";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, CHAIN_ID } from "./config";
import { createAndFinalizeBlock, describeWithIce, customRequest } from "./util";

// We use ethers library in this test as apparently web3js's types are not fully EIP-1559 compliant yet.
describeWithIce("Ice RPC (Max Priority Fee Per Gas)", (context) => {
	async function sendTransaction(context, payload: any) {
		let signer = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, context.ethersjs);
		// Ethers internally matches the locally calculated transaction hash against the one returned as a response.
		// Test would fail in case of mismatch.
		const tx = await signer.sendTransaction(payload);
		return tx;
	}

	let nonce = 0;

	async function createBlocks(block_count, priority_fees) {
		const gasfee = context.ethersjs.getGasPrice();
		for (var b = 0; b < block_count; b++) {
			for (var p = 0; p < priority_fees.length; p++) {
				await sendTransaction(context, {
					from: GENESIS_ACCOUNT,
					to: "0x0000000000000000000000000000000000000000",
					data: "0x",
					value: "0x00",
					maxFeePerGas: gasfee,
					maxPriorityFeePerGas: context.web3.utils.numberToHex(priority_fees[p]),
					accessList: [],
					nonce: nonce,
					gasLimit: gasfee,
					chainId: CHAIN_ID,
				});
				nonce++;
			}
			// await createAndFinalizeBlock(context.web3);
		}
	}

	step("should default to zero on genesis", async function () {
		let result = await customRequest(context.web3, "eth_maxPriorityFeePerGas", []);
		expect(result.result).to.be.eq("0x0");
	});

	step("should default to zero on empty blocks", async function () {
		// await createAndFinalizeBlock(context.web3);
		let result = await customRequest(context.web3, "eth_maxPriorityFeePerGas", []);
		expect(result.result).to.be.eq("0x0");
	});

	// If in the last 20 blocks at least one is empty (or only contains zero-tip txns), the
	// suggested tip will be zero.
	// That's the expected behaviour in this simplified oracle version: there is a decent chance of
	// being able to include a zero-tip txn in a low congested network.
	step("maxPriorityFeePerGas should suggest zero if there are recent empty blocks", async function () {
		this.timeout(100000);

		for (let i = 0; i < 10; i++) {
			await createBlocks(1, [0, 1, 2, 3, 4, 5]);
		}
		// await createAndFinalizeBlock(context.web3);
		for (let i = 0; i < 9; i++) {
			await createBlocks(1, [0, 1, 2, 3, 4, 5]);
		}

		let result = (await customRequest(context.web3, "eth_maxPriorityFeePerGas", [])).result;
		expect(result).to.be.eq("0x0");
	});
});
