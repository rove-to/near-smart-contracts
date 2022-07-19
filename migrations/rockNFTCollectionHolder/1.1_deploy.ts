import * as dotenv from 'dotenv';
dotenv.config();

import {RockNFTCollectionHolder} from "./rockNFTCollectionHolder";
import * as fs from "fs";

(async () => {
    try {
        const nft = new RockNFTCollectionHolder(process.env.NETWORK);
        if (!nft.config) {
            console.log("wrong network");
            return;
        }
        const wasm = process.argv[2];
        const contractAccountId = process.argv[3];
        const adminId = process.argv[4] || process.env.ADMIN_ID || "";
        const operatorId = process.argv[5] || process.env.OPERATOR_ID || "";
        const treasuryId = process.argv[6] || process.env.TREASURY_ID || "";
        const initIMOFee = process.argv[7] || process.env.INIT_IMO_FEE || "";
        const rockPurchaseFee = process.argv[8] || process.env.ROCK_PURCHASE_FEE || "";
        const init_imo_nft_holder_size = process.argv[9] || "";
        const contractMetadataFile = process.argv[10] || "";

        if (contractMetadataFile == "") {
            console.log("missing contract metadata file")
        }
        const contractMetadata = await JSON.parse((await fs.readFileSync(contractMetadataFile)).toString());

        await nft.deploy(wasm, contractAccountId, adminId, operatorId, treasuryId,
            initIMOFee,
            parseInt(rockPurchaseFee, 10),
            parseInt(init_imo_nft_holder_size, 10),
            contractMetadata,
            true);
        console.log("Deployed contract on contractAccountId:", contractAccountId)
    } catch (e) {
        console.log(e);
    }
})();
