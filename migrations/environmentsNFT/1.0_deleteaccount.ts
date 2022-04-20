import * as dotenv from 'dotenv';

dotenv.config();

import {EnvironmentNFT} from "./environmentNFT";

(async () => {
    try {
        if (process.env.NETWORK != "testnet") {
            console.log("wrong network");
            return;
        }
        const nft = new EnvironmentNFT(process.env.NETWORK);
        const deletedAccount = process.argv[2];
        await nft.deleteAccount(deletedAccount);
        console.log("Deleted contractAccountID", deletedAccount);
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();