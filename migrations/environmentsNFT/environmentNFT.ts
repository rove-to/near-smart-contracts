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

    async createAccount(newAccountId: string, amount: string) {
        const near = await connect(this.config);
        const creatorAccount = await near.account(process.env.CREATOR_ACCOUNT_ID);
        const keyPair = KeyPair.fromRandom("ed25519");
        const publicKey = keyPair.publicKey.toString();

        const newAccount = await creatorAccount.functionCall({
            contractId: this.network,
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
            await deletedAccount.deleteAccount(process.env.CREATOR_ACCOUNT_ID);
        } catch (e) {
            console.log(e);
        }
    }

    async deleteKey(deletedAccountID: string) {
        this.near = await connect(this.config);
        try {
            await this.config.keyStore.removeKey(this.config.networkId, deletedAccountID);
        } catch (e) {
            console.log(e);
        }
    }

    async deploy(wasmFile: string, contractAccountID: string, price: number, tokenMetadata: any, adminID: string, operatorID: string, treasuryID: string) {
        console.log("wasm: ", wasmFile);
        this.near = await connect(this.config);

        // create contract account id
        console.log("contractAccountID:", contractAccountID);
        try {
            const contractAccount = await this.near.account(contractAccountID);
            const response = await contractAccount.deployContract(fs.readFileSync(wasmFile));
            console.log("deploy on:", response.transaction.hash);

            // call init func
            await this.init(contractAccountID, contractAccount, adminID, operatorID, treasuryID, price, tokenMetadata);
        } catch (e) {
            console.log(e);
        }
    }

    async init(contractAccountId: string, account: any, adminId: string, operatorId: string, treasuryId: string, price: number, tokenMetadata: any) {
        const contract = new nearAPI.Contract(account, contractAccountId, {
            viewMethods: ['nft_metadata'],
            changeMethods: ['new'],
        });
        const args = {
            admin_id: adminId,
            operator_id: operatorId,
            treasury_id: treasuryId,
            max_supply: 20,
            metadata: {spec: "nft-1.0.0", name: "rove-nft", symbol: "ROVE-NFT"},
            token_price: 1,
            token_metadata: tokenMetadata
        };
        await contract.new(args, "300000000000000");
    }

    async createNFT(contractAccountId: string, signerId: string, receiverId: string, attachedDeposit: string) {
        this.near = await connect(this.config);
        console.log({contractAccountId});
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ["nft_create"]
            });

            await contract.nft_create(
                {
                    receiver_id: receiverId
                },
                "300000000000000",
                attachedDeposit
            );
        } catch (e) {
            console.log(e);
        }
    }
}

export {
    EnvironmentNFT
};