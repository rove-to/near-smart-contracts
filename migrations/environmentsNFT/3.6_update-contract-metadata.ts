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
        const contractMetadataFile = process.argv[4];
        const contractMetadata = JSON.parse((await fs.readFileSync(contractMetadataFile)).toString());

        console.log({contractAccountId, signerId, contractMetadata});
        if (!contractAccountId || !signerId ) {
            throw new Error("invalid arguments")
        }

        const result = await nft.updateContractMetadata(contractAccountId, signerId, contractMetadata);

        console.log({result});
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();