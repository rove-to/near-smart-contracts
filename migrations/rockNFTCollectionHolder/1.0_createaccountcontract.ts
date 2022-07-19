import * as dotenv from 'dotenv';
dotenv.config();

import {RockNFTCollectionHolder} from "./rockNFTCollectionHolder";

(async () => {
    try {
        const nft = new RockNFTCollectionHolder(process.env.NETWORK);
        if (!nft.config) {
            console.log("wrong network");
            return;
        }

        const contractAccountId = `${process.argv[2]}-contract-${(new Date()).getTime()}-${process.env.CREATOR_ACCOUNT_ID}`;
        const depositAmount = process.argv[3];
        await nft.createAccount(contractAccountId, depositAmount);
        console.log("Created contractAccountID:%s with deposit %s", contractAccountId, depositAmount);
    } catch (e) {
        console.log(e);
    }
})();
