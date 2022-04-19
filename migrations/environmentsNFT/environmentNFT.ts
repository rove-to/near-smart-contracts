// import * as fs from "fs";
// import {connect} from "near-api-js";

// const nearConfig = require("../../near.config");

class EnvironmentNFT {
    network: string;

    constructor(network: any) {
        this.network = network;
    }

    async deploy(wasm: string, accountID: string) {
        console.log(wasm);
        console.log(accountID);
        // let near
        // if (this.network == "testnet") {
        //     // near  = await connect(nearConfigTest);
        // } else {
        //     near = await connect(nearConfig);
        // }
        // const account = await near.account(process.env.NFT_CONTRACT_ID);
        // const response = await account.deployContract(fs.readFileSync(wasm));
        // console.log(response);
    }

    async init(wasm: string) {
        // asfasf
    }
}

export {EnvironmentNFT};