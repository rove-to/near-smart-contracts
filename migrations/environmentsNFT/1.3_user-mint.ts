import * as dotenv from 'dotenv';

dotenv.config();

import {EnvironmentNFT} from "./environmentNFT";
import * as fs from "fs";

(async () => {
    try {
        if (process.env.NETWORK != "testnet") {
            console.log("wrong network");
            return;
        }
        const nft = new EnvironmentNFT(process.env.NETWORK);

        const contractAccountId = process.argv[2];
        const signerId = process.argv[3];
        const nftTypeId = process.argv[4];
        const receiverId = process.argv[5];
        const attachedDeposit = process.argv[6];

        console.log({contractAccountId, signerId});
        if (!contractAccountId || !signerId) {
            throw new Error("invalid arguments")
        }

        await nft.userMint(contractAccountId, signerId, nftTypeId, receiverId, attachedDeposit);

        console.log("User mint nft", {contractAccountId, signerId});
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();