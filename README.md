# NEAR smart contract

## Build the contract

```bash
cd ./contracts/goods/environments
bash build.sh
```

After this step, a `.wasm` file is located in `./out`

## Deploy the contract

Please follow the doc: https://docs.near.org/docs/tutorials/contracts/nfts/minting#deploy-the-contract
(skip `yarn build` command)

## Manually test

### To create a nft with price 1 NEAR

```near call $NFT_CONTRACT_ID create_nft '{"token_id": "token-2", "receiver_id": "'$NFT_CONTRACT_ID'", "token_metadata": {"title": "My Non Fungible Team Token", "description": "The Team Most Certainly Goes :)", "media": "https://bafybeiftczwrtyr3k7a2k4vutd3amkwsmaqyhrdzlhvpt33dyjivufqusq.ipfs.dweb.link/goteam-gif.gif"}, "price_in_string": "1000000000000000000000000"}' --accountId $NFT_CONTRACT_ID --amount 0.1```

### User dacdodinh99.testnet want to mint created token, he/she call

```near call $NFT_CONTRACT_ID user_mint '{"receiver_id": "dacdodinh99.testnet"}' --accountId dacdodinh99.testnet --amount 0.1```