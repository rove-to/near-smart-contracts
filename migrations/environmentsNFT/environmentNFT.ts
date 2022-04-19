import * as fs from "fs";
import {connect} from "near-api-js";

const nearConfig = require("../../near.config");

class EnvironmentNFT {
    network: string;
    near: any;

    constructor(network: any) {
        this.network = network;
    }

    async deploy(wasm: string, accountID: string) {
        console.log(wasm);
        console.log(accountID);
        this.near = await connect(nearConfig[this.network]);

        // const account = await near.account(accountID);
        // const response = await account.deployContract(fs.readFileSync(wasm));
        // console.log(response);
    }

    async init(wasm: string) {
        // asfasf
    }
}

export {EnvironmentNFT};