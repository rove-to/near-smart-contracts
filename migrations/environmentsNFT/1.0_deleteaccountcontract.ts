import * as dotenv from 'dotenv';

dotenv.config();

import {EnvironmentNFT} from "./environmentNFT";

(async () => {
    try {
        const nft = new EnvironmentNFT(process.env.NETWORK);
        if (!nft.config) {
            console.log("wrong network");
            return;
        }
        const deletedAccount = process.argv[2];
        await nft.deleteAccount(deletedAccount);
        console.log("Deleted contractAccountID", deletedAccount);
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();
