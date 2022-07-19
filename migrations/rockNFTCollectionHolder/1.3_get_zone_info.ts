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
        const metaverseID = process.argv[2] || "";
        const zoneIndex = process.argv[3];
        const contractAccountId = process.argv[4];
        const signerAccountId = process.argv[5];

        await nft.getZoneInfo(signerAccountId, contractAccountId, metaverseID, parseInt(zoneIndex));

    } catch (e) {
        console.log(e);
    }
})();

