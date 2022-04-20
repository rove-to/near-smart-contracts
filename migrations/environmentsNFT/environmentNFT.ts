import * as fs from "fs";

const nearAPI = require("near-api-js");
const {connect, utils, KeyPair} = nearAPI;

const nearConfig = require("../../near.config");

class EnvironmentNFT {
    network: string;
    near: any;
    config: any;

    constructor(network: any) {
        this.network = network;
        this.config = nearConfig[this.network];
        console.log("network:", this.network);
        console.log("config:", this.config);
    }

    async createAccount(creatorAccountId: any, newAccountId: string, amount: string) {
        const near = await connect(this.config);
        const creatorAccount = await near.account(creatorAccountId);
        const keyPair = KeyPair.fromRandom("ed25519");
        const publicKey = keyPair.publicKey.toString();

        const newAccount = await creatorAccount.functionCall({
            contractId: "testnet",
            methodName: "create_account",
            args: {
                new_account_id: newAccountId,
                new_public_key: publicKey,
            },
            gas: "300000000000000",
            attachedDeposit: utils.format.parseNearAmount(amount),
        });
        await this.config.keyStore.setKey(this.config.networkId, newAccountId, keyPair);
        return newAccount;
    }

    async deleteAccount(deletedAccountID: string) {
        this.near = await connect(this.config);
        const deletedAccount = await this.near.account(deletedAccountID);
        try {
            await this.config.keyStore.removeKey(deletedAccountID);
            await deletedAccount.deleteAccount(process.env.ACCOUNT_ID);
        } catch (e) {
            console.log(e);
        }
    }

    async deploy(wasmFile: string, contractAccountID: string, depositAmountContract: string,
                 price: number, tokenMetadata: any, adminID: string, operatorID: string, treasuryID: string) {
        console.log("wasm: ", wasmFile);
        this.near = await connect(this.config);

        // create contract account id
        console.log("contractAccountID:", contractAccountID);
        try {
            await this.createAccount(process.env.ACCOUNT_ID, contractAccountID, depositAmountContract)
            const contractAccount = await this.near.account(contractAccountID);
            console.log("create success contract account:", contractAccount);
            const response = await contractAccount.deployContract(fs.readFileSync(wasmFile));
            console.log("deploy on:", response.transaction.hash);

            // call init func
            await this.init(adminID, operatorID, price, tokenMetadata);
        } catch (e) {
            console.log(e);
        }
    }

    async init(adminID: string, operatorID: string, price: number, tokenMetadata: any) {

    }

    async createNFT(receiverID: string) {

    }
}

export {
    EnvironmentNFT
};