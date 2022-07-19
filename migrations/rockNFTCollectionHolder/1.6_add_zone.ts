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
        const typeZone = process.argv[2] || "";
        const zoneIndex = process.argv[3] || "";
        const collectionAddress = process.argv[4] || "";
        const rockIndexFrom = process.argv[5] || "";
        const rockIndexTo = process.argv[6] || "";
        const price = process.argv[7] || process.env.PUBLIC_PRICE || "";
        const signerAccount = process.argv[8] || process.env.SIGNER_ACCOUNT || "";
        const contractAccount = process.argv[9] || process.env.CONTRACT_ACCOUNT || "";
        const metaverseID = process.argv[10] || process.env.METAVERSE_ID || "";
        const attachDeposit = process.argv[11] || process.env.DEPOSIT || "";

        await nft.addZone(signerAccount, contractAccount, metaverseID, parseInt(zoneIndex),
            parseInt(typeZone, 10),
            parseInt(rockIndexFrom, 10),
            parseInt(rockIndexTo, 10),
            price,
            collectionAddress, attachDeposit);
    } catch (e) {
        console.log(e);
    }
})();

