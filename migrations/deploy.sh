echo "$NFT_CONTRACT_ID"
near deploy --wasmFile out/main.wasm --accountId "$NFT_CONTRACT_ID"
