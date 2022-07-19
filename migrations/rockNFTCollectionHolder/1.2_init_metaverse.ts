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
        const totalSupply = process.argv[2] || process.env.PUBLIC_TOTAL_SUPPLY || "";
        const price = process.argv[3] || process.env.PUBLIC_PRICE || "";
        const collectionAddress =  process.argv[4] || "";
        const signerAccount = process.argv[5] || process.env.SIGNER_ACCOUNT || "";
        const contractAccount = process.argv[6] || process.env.CONTRACT_ACCOUNT || "";
        const metaverseID = process.argv[7] || process.env.METAVERSE_ID || "";
        const attachDeposit = process.argv[8] || process.env.DEPOSIT || "";

        await nft.initMetaverse(signerAccount, contractAccount, metaverseID, parseInt(totalSupply), price,
            collectionAddress, attachDeposit);

    } catch (e) {
        console.log(e);
    }
})();

