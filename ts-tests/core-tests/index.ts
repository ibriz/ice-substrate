import { SnowApi } from "./services";

SnowApi.initialize().then(res => {
    console.log(SnowApi.wallets?.ALICE_STASH.address);
    SnowApi.cleanUp();
});
