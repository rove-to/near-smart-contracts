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
        console.log(process.argv);
        const nft = new EnvironmentNFT(process.env.NETWORK);
        const wasm = process.argv[2];
        const contractAccountId = process.argv[3];
        const tokenMetadataFile = process.argv[4];
        const tokenMetadata = JSON.parse((await fs.readFileSync(tokenMetadataFile)).toString());
        const adminId = process.argv[5] || process.env.ADMIN_ID || "";
        const operatorId = process.argv[6] || process.env.OPERATOR_ID || "";
        const treasuryId = process.argv[7] || process.env.TREASURY_ID || "";
        await nft.deploy(wasm, contractAccountId, 0, tokenMetadata, adminId, operatorId, treasuryId);
        console.log("Deployed contract on contractAccountId:", contractAccountId)
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();