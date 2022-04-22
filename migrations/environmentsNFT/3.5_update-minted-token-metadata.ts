import * as dotenv from 'dotenv';

dotenv.config();

import {EnvironmentNFT} from "./environmentNFT";
import * as fs from 'fs';
(async () => {
    try {
        if (process.env.NETWORK != "testnet") {
            console.log("wrong network");
            return;
        }
        const nft = new EnvironmentNFT(process.env.NETWORK);

        const contractAccountId = process.argv[2];
        const signerId = process.argv[3];
        const tokenId = process.argv[4];
        const tokenMetadataFile = process.argv[5];
        const tokenMetadata = JSON.parse((await fs.readFileSync(tokenMetadataFile)).toString());

        console.log({contractAccountId, signerId, tokenId, tokenMetadata});
        if (!contractAccountId || !signerId ) {
            throw new Error("invalid arguments")
        }

        const result = await nft.updateMintedTokenMetadata(contractAccountId, signerId, tokenId, tokenMetadata);

        console.log({result});
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();