import { step } from "mocha-steps";
import chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { describeWithContext } from "../utils";
import BigNumber from "bignumber.js";
import { createAsset, mintAsset, getAssetBalance, transferAsset } from "./helper";
import { SnowApi } from "../../services";

chai.use(chaiAsPromised);

const { expect } = chai;

const STEP_TIMEOUT = 40_000;

describeWithContext("\n\n👉 Tests for asset creation/transfer/destruction", (context) => {
	const assetId = 0;
	const minBalance = new BigNumber(1000000); // 10^6

	step("🌟 Asset can be created", async function (done) {
		try {
			this.timeout(STEP_TIMEOUT);
			console.log("\nCreating an asset...\n");

			// Bob creates asset, but Bob_stash is the admin
			await createAsset(
				SnowApi.wallets!.BOB,
				SnowApi.api!,
				assetId,
				SnowApi.wallets!.BOB_STASH.address,
				minBalance,
			);
			done();
		} catch (err) {
			done(err);
		}
	});

	step("🌟 Asset can be minted only by the admin", async function (done) {
		try {
			this.timeout(STEP_TIMEOUT);

			// Admin Bob_stash mints assets to Alice
			await mintAsset(
				SnowApi.wallets!.BOB_STASH,
				SnowApi.api!,
				assetId,
				SnowApi.wallets!.ALICE.address,
				new BigNumber(Math.pow(10, 10)),
			);

			const aliceAssets = await getAssetBalance(SnowApi.api!, assetId, SnowApi.wallets!.ALICE.address);
			expect(aliceAssets).to.equal(new BigNumber(Math.pow(10, 10)).toFixed(0));

			// Non-admin Bob cannot mint assets
			expect(
				mintAsset(
					SnowApi.wallets!.BOB,
					SnowApi.api!,
					assetId,
					SnowApi.wallets!.ALICE.address,
					new BigNumber(Math.pow(10, 10)),
				),
			)
				.to.be.rejectedWith(/account has no permission/, "Only admin should be allowed to mint asset")
				.notify(done);
		} catch (err) {
			done(err);
		}
	});

	step("🌟 Asset can be transferred", async function (done) {
		try {
			this.timeout(STEP_TIMEOUT);
			const transferAmount = new BigNumber(Math.pow(10, 10)).minus(minBalance).plus(1);

			// transfer balance from Alice to Alice_stash, such that balance on Alice drops just below the minBalance
			await transferAsset(
				SnowApi.wallets!.ALICE,
				SnowApi.api!,
				assetId,
				SnowApi.wallets!.ALICE_STASH.address,
				transferAmount,
			);

			const aliceStashAssets = await getAssetBalance(SnowApi.api!, assetId, SnowApi.wallets!.ALICE_STASH.address);
			const aliceAssets = await getAssetBalance(SnowApi.api!, assetId, SnowApi.wallets!.ALICE.address);

			// alice balance is zero since it dropped below minBalance
			expect(aliceAssets).to.equal("0");
			// aliceStash gets all the Alice balance since her balance dropped below minBalance after transfer
			expect(aliceStashAssets).to.equal(Math.pow(10, 10).toString());

			expect(
				transferAsset(
					SnowApi.wallets!.ALICE_STASH,
					SnowApi.api!,
					assetId,
					SnowApi.wallets!.ALICE.address,
					minBalance.minus(1),
				),
			)
				.to.be.rejectedWith(/BelowMinimum/, "Assets below minBalance should not be transferred")
				.notify(done);
		} catch (err) {
			done(err);
		}
	});

	step("🌟 Asset can be destroyed", async function (done) {
		try {
			this.timeout(STEP_TIMEOUT);

			
		} catch (err) {
			console.log(err);
		}
	});
});
