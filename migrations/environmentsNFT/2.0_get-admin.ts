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

        const contractAccountId = process.argv[2];
        const signerId = process.argv[3];

        console.log({contractAccountId, signerId});
        if (!contractAccountId || !signerId ) {
            throw new Error("invalid arguments")
        }

        const result = await nft.get("get_admin", contractAccountId, signerId);

        console.log({result});
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();