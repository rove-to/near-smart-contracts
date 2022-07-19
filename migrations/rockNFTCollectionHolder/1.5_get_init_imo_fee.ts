import * as dotenv from 'dotenv';
dotenv.config();

import {RockNFTCollectionHolder as RockNFT} from "./rockNFTCollectionHolder";

(async () => {
    try {
        const nft = new RockNFT(process.env.NETWORK);
        if (!nft.config) {
            console.log("wrong network");
            return;
        }
        const contractAccountId = process.argv[2];
        const signerAccountId = process.argv[3];

        await nft.getIMOInitFee(signerAccountId, contractAccountId);

    } catch (e) {
        console.log(e);
    }
})();

