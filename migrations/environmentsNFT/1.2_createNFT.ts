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
        const receiverId = process.argv[4];
        const attachedDeposit = process.argv[5];

        console.log({contractAccountId, signerId, receiverId, attachedDeposit});
        if (!!contractAccountId || !signerId || !receiverId || !attachedDeposit) {
            throw new Error("invalid arguments")
        }

        await nft.createNFT(contractAccountId, signerId, receiverId, attachedDeposit);

        console.log("Created nft", {contractAccountId, signerId, receiverId, attachedDeposit});
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();