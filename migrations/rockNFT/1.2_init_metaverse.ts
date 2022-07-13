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
        const totalSupply = process.argv[2] || process.env.PUBLIC_TOTAL_SUPPLY || "";
        const price = process.argv[3] || process.env.PUBLIC_PRICE || "";
        const signerAccount = process.argv[4] || process.env.SIGNER_ACCOUNT || "";
        const contractAccount = process.argv[5] || process.env.CONTRACT_ACCOUNT || "";
        const metaverseID = process.argv[6] || process.env.METAVERSE_ID || "";
        const attachDeposit = process.argv[7] || process.env.DEPOSIT || "";

        await nft.initMetaverse(signerAccount, contractAccount, metaverseID, parseInt(totalSupply), price, attachDeposit);

    } catch (e) {
        console.log(e);
    }
})();

