import * as dotenv from 'dotenv';
dotenv.config();

import {RockNFT} from "./rockNFT";

(async () => {
    try {
        const nft = new RockNFT(process.env.NETWORK);
        if (!nft.config) {
            console.log("wrong network");
            return;
        }
        const wasm = process.argv[2];
        const contractAccountId = process.argv[3];
        await nft.deploy(wasm, contractAccountId, "", "", "",
            "", 0, "",
            false);
        console.log("Upgraded contract on contractAccountId:", contractAccountId)
    } catch (e) {
        console.log(e);
    }
})();
