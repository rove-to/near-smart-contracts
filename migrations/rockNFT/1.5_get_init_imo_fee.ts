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
        const contractAccountId = process.argv[2];
        const signerAccountId = process.argv[3];

        await nft.get("get_init_imo_fee", contractAccountId, signerAccountId);

    } catch (e) {
        console.log(e);
    }
})();

