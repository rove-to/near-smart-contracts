module.exports = {
    testnet: {
        networkId: "testnet",
        keyStore: "~/.near-credentials/testnet/test-1.testnet.json", // optional if not signing transactions
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://wallet.testnet.near.org",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://explorer.testnet.near.org",
    },
    mainnet: {
        networkId: "mainnet",
        keyStore, // optional if not signing transactions
        nodeUrl: "https://rpc.mainnet.near.org",
        walletUrl: "https://wallet.mainnet.near.org",
        helperUrl: "https://helper.mainnet.near.org",
        explorerUrl: "https://explorer.mainnet.near.org",
    },
    betanet: {
        networkId: "betanet",
        keyStore, // optional if not signing transactions
        nodeUrl: "https://rpc.betanet.near.org",
        walletUrl: "https://wallet.betanet.near.org",
        helperUrl: "https://helper.betanet.near.org",
        explorerUrl: "https://explorer.betanet.near.org",
    },
    localnet: {
        networkId: "local",
        nodeUrl: "http://localhost:3030",
        walletUrl: "http://localhost:4000/wallet",
    }
}
