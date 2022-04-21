import * as dotenv from 'dotenv';

dotenv.config();

import {EnvironmentNFT} from "./environmentNFT";
import * as fs from "fs";
import {enums} from "near-api-js/lib/utils";

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
        console.log(process.env);
        const adminId = process.argv[5] || process.env.ADMIN_ID || "";
        const operatorId = process.argv[6] || process.env.OPERATOR_ID || "";
        const treasuryId = process.argv[7] || process.env.TREASURY_ID || "";
        const tokenPrice = process.argv[8] || process.env.TOKEN_PRICE || 0;
        const maxSupply = process.argv[9] || process.env.TOKEN_PRICE || 0;
        const contractMetadataFile = process.argv[10];
        const contractMetadata = JSON.parse((await fs.readFileSync(contractMetadataFile)).toString());
        await nft.deploy(wasm, contractAccountId, tokenPrice, tokenMetadata, adminId, operatorId, treasuryId, maxSupply, contractMetadata);
        console.log("Deployed contract on contractAccountId:", contractAccountId)
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();
