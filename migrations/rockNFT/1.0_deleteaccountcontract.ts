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
        const deletedAccount = process.argv[2];
        await nft.deleteAccount(deletedAccount);
        console.log("Deleted contractAccountID", deletedAccount);
    } catch (e) {
        console.log(e);
    }
})();
