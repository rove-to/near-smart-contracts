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
        nft.deploy(process.argv[2], process.argv[3]);
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();