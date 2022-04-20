# NEAR smart contract

# Swicth network

``echo NEAR_ENV=networkID``

# Clone .env

``cp .env.local .env``

## Build rust contract
- `yarn compile`
- `yarn compile:clear`

## Deploy contract
### 1. Create contract account 

``yarn ts:run ./path_to_ts_migrate_file/1.0_createaccountcontract.ts contract_name account_contractId deposit_amount``

### 2. Deploy with contract account

``yarn ts:run ./path_to_ts_migrate_file/1.1_deploy.ts [args...]``