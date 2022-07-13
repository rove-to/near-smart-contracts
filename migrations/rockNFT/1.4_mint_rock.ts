import * as dotenv from 'dotenv';
dotenv.config();

import {RockNFT} from "./rockNFT";
import * as fs from "fs";

(async () => {
    try {
        const nft = new RockNFT(process.env.NETWORK);
        if (!nft.config) {
            console.log("wrong network");
            return;
        }
        const metaverseID = process.argv[2] || "";
        const zoneIndex = process.argv[3];
        const rockIndex = process.argv[4];
        const contractAccountId = process.argv[5];
        const signerAccountId = process.argv[6];
        const tokenMetadataFile = process.argv[7];
        const tokenMetadata = JSON.parse((await fs.readFileSync(tokenMetadataFile)).toString());
        const receiverId = process.argv[8];
        const attachDeposit = process.argv[9];

        await nft.mintRock(signerAccountId, contractAccountId, metaverseID, parseInt(zoneIndex), parseInt(rockIndex),
            receiverId, tokenMetadata, attachDeposit
            );
    } catch (e) {
        console.log(e);
    }
})();

