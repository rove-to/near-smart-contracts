import * as dotenv from 'dotenv';

dotenv.config();

import {EnvironmentNFT} from "./environmentNFT";

(async () => {
    try {
        if (process.env.NETWORK != "testnet") {
            console.log("wrong network");
            return;
        }
        const nft = new EnvironmentNFT(process.env.NETWORK);
        const wasm = process.argv[2];
        const contractAccountId = process.argv[3];
        const depositAmount = process.argv[4];
        nft.deploy(wasm, contractAccountId, depositAmount, 0, "", "", "", "");
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();