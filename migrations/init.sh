near call "$NFT_CONTRACT_ID" new \
'{"admin_id": "'$ADMIN_ID'", "operator_id": "'$OPERATOR_ID'", "treasury_id": "'$NFT_TREASURY_ID'", "metadata": {"spec": "nft-1.0.0", "name": "rove-nft", "symbol" : "ROVE-NFT"}, "max_supply": '$MAX_SUPPLY'}' \
--accountId $NFT_CONTRACT_ID
