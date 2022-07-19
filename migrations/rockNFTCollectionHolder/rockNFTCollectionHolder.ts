import * as fs from "fs";

const nearAPI = require("near-api-js");
const {connect, utils, KeyPair} = nearAPI;
const nearConfig = require("../../near.config");

class RockNFTCollectionHolder {
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

    async deploy(wasmFile: string, contractAccountID: string,
                 adminID: string, operatorID: string, treasuryID: string,
                 initIMOFee: string, rock_purchase_fee: number,
                 initIMONFTHolderSize: number,
                 contractMetadata: any,
                 isRunInit: boolean
    ) {
        this.near = await connect(this.config);
        try {
            const contractAccount = await this.near.account(contractAccountID);
            const response = await contractAccount.deployContract(fs.readFileSync(wasmFile));
            console.log("deploy on:", response.transaction.hash);

            if (isRunInit) {
                await this.init(contractAccountID, contractAccount, adminID, operatorID, treasuryID,
                    initIMOFee, rock_purchase_fee, initIMONFTHolderSize,
                    contractMetadata);
            }
        } catch (e) {
            console.log(e);
        }
    }

    async initMetaverse(signerAccountId: string, contractAccountID: string, metaverseID: string, totalSupply: number,
                        price: string, collectionAddress: string, attachedDeposit: string) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerAccountId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountID,  {
               viewMethods: [],
                changeMethods: ["init_metaverse"],
            });
            const args = {
                metaverse_id: metaverseID,
                _zone2: {
                    zone_index: 2,
                    price: utils.format.parseNearAmount(price),
                    core_team_addr: '',
                    collection_addr: collectionAddress,
                    type_zone: 2,
                    rock_index_from: 2,
                    rock_index_to: 501, // contract dont use this argument
                }
            }
            console.log("call initMetaverse with args", args);
            await contract.init_metaverse({args, gas: "300000000000000",
                amount: utils.format.parseNearAmount(attachedDeposit)}
            );
        } catch (e) {
            console.log(e);
        }
    }

    async getZoneInfo(signerAccountId: string, contractAccountID: string, metaverseID: string, zoneIndex: number) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerAccountId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountID,  {
                viewMethods: ["get_zone_info"],
                changeMethods: [],
            });
            const args = {
                metaverse_id: metaverseID,
                zone_index: zoneIndex,
            }
            console.log("call get_zone_info with args", args);
            const response = await contract.get_zone_info(args);
            console.log(`get_zone_info response: ${response}`);
        } catch (e) {
            console.log(e);
        }
    }

    async init(contractAccountId: string, account: any, adminId: string, operatorId: string, treasuryId: string,
               init_imo_fee: string, rock_purchase_fee: number,
               init_imo_nft_holder_size: number,
               contractMetadata: any) {
        const contract = new nearAPI.Contract(account, contractAccountId, {
            viewMethods: [],
            changeMethods: ['new'],
        });
        const args = {
            admin_id: adminId,
            operator_id: operatorId,
            treasury_id: treasuryId,
            init_imo_fee: init_imo_fee,
            rock_purchase_fee: rock_purchase_fee,
            init_imo_nft_holder_size,
            metadata: contractMetadata,
        };
        console.log(args);
        await contract.new({args, gas: "300000000000000"});
    }

    async mintRock(signerId: string, contractAccountId: string, metaverseId: string, zoneIndex: number, rockIndex: number,
                   receiverId: string, tokenMetadata: any, attachedDeposit: string) {
        this.near = await connect(this.config);
        console.log({contractAccountId});
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ["mint_rock"]
            });

            const args =  {
                metaverse_id: metaverseId,
                zone_index: zoneIndex,
                rock_index: rockIndex,
                receiver_id: receiverId,
                token_metadata: tokenMetadata,
            }

            console.log(`call mint_rock with args:`, args)
            const response = await contract.mint_rock(
                {
                    args,
                    gas: "300000000000000",
                    amount: utils.format.parseNearAmount(attachedDeposit),
                },
            );
            console.log(`response mint_rock ${response}`)
        } catch (e) {
            console.log(e);
        }
    }

    async get(method: string, contractAccountId: string, signerId: string) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: ["get_admin"],
                changeMethods: []
            });
            const response = await contract.get_admin({});
            return response;
        } catch (e) {
            console.log(e);
        }
    }

    async setAdmin(contractAccountId: string, signerId: string, newAdminId: string) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ['change_admin']
            });
            const response = await contract.change_admin({
                args: {
                    new_admin_id: newAdminId
                },
                amount: "1"
            });
        } catch (e) {
            console.log(e);
        }
    }

    async changeOperator(contractAccountId: string, signerId: string, newOperatorId: string) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ['change_operator']
            });
            const response = await contract.change_operator({
                args: {
                    new_operator_id: newOperatorId
                },
                amount: "1"
            });
        } catch (e) {
            console.log(e);
        }
    }

    async changeTreasury(contractAccountId: string, signerId: string, newTreasuryId: string) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ['change_treasury']
            });
            const response = await contract.change_treasury({
                args: {
                    new_treasury_id: newTreasuryId
                },
                amount: "1"
            });
        } catch (e) {
            console.log(e);
        }
    }

    async updateTokenPrice(contractAccountId: string, signerId: string, newTokenPrice: string) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ['update_token_price']
            });
            const response = await contract.update_token_price({
                args: {
                    updated_price_in_string: utils.format.parseNearAmount(newTokenPrice)
                },
                amount: "1"
            });
        } catch (e) {
            console.log(e);
        }
    }

    async updateTokenMetadata(contractAccountId: string, signerId: string, newTokenMetadata: string) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ['update_token_metadata']
            });
            const response = await contract.update_token_metadata({
                args: {
                    updated_token_metadata: newTokenMetadata
                },
                amount: "1"
            });
        } catch (e) {
            console.log(e);
        }
    }

    async updateMintedTokenMetadata(contractAccountId: string, signerId: string, tokenId: string, newTokenMetadata: any) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ['update_minted_token_metadata']
            });
            const response = await contract.update_minted_token_metadata({
                args: {
                    updated_token_metadata: newTokenMetadata,
                    token_id: tokenId
                },
                amount: "1"
            });
        } catch (e) {
            console.log(e);
        }
    }

    async updateContractMetadata(contractAccountId: string, signerId: string, newContractMetadata : any) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ['update_contract_metadata']
            });
            const response = await contract.update_contract_metadata({
                args: {
                    updated_contract_metadata: newContractMetadata,
                },
                amount: "1"
            });
        } catch (e) {
            console.log(e);
        }
    }

    async updateRoyalties(contractAccountId: string, signerId: string, nftTypeId: string, royaltyId : string, royaltyAmount: number) {
        this.near = await connect(this.config);
        try {
            const signerAccount = await this.near.account(signerId);
            const contract = new nearAPI.Contract(signerAccount, contractAccountId, {
                viewMethods: [],
                changeMethods: ['update_royalties']
            });
            const updated_royalties = {} as any;
            updated_royalties[royaltyId] = royaltyAmount;
            const response = await contract.update_royalties({
                args: {
                    nft_type_id: nftTypeId,
                    updated_royalties
                },
                amount: "1"
            });
            console.log(response);
        } catch (e) {
            console.log(e);
        }
    }
}

export {
    RockNFTCollectionHolder
};
