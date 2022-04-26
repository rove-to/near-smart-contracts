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
        const price = process.argv[5];
        const tokenMetadataFile = process.argv[6];
        const tokenMetadata = JSON.parse((await fs.readFileSync(tokenMetadataFile)).toString());
        const maxSupply = parseInt(process.argv[7]);
        const attachedDeposit = process.argv[8];

        console.log({contractAccountId, signerId});
        if (!contractAccountId || !signerId) {
            throw new Error("invalid arguments")
        }

        await nft.createNft(contractAccountId, signerId, nftTypeId, price, tokenMetadata, maxSupply, attachedDeposit);

        console.log("Created nft", {contractAccountId, signerId});
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();
