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
        const deletedAccount = process.argv[2];
        await nft.deleteAccount(deletedAccount);
        console.log("Deleted contractAccountID", deletedAccount);
    } catch (e) {
        console.log(e);
    }
})();
