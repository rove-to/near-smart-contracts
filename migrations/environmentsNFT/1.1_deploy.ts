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
        const adminId = process.argv[4] || process.env.ADMIN_ID || "";
        const operatorId = process.argv[5] || process.env.OPERATOR_ID || "";
        const treasuryId = process.argv[6] || process.env.TREASURY_ID || "";
        const contractMetadataFile = process.argv[7];
        const contractMetadata = JSON.parse((await fs.readFileSync(contractMetadataFile)).toString());
        await nft.deploy(wasm, contractAccountId, adminId, operatorId, treasuryId, contractMetadata,
            true);
        console.log("Deployed contract on contractAccountId:", contractAccountId)
    } catch (e) {
        // Deal with the fact the chain failed
        console.log(e);
    }
})();
