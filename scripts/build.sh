[ ! -d "/src/ethereum" ] && mkdir ./src/ethereum
[ ! -d "/src/ethereum/abi" ] && mkdir ./src/ethereum/abi

# Make the smart contract ABI available to our Typescript sources
cp -R ./node_modules/@hoprnet/hopr-ethereum/build/extracted/abis/*.json ./src/ethereum/abi;

# Make our smart contract addresses available to our Typescript sources
cp ./node_modules/@hoprnet/hopr-ethereum/build/lib/scripts/addresses.* ./src/ethereum

# Compile our Typescript sources
yarn run tsc;

# Copy generated TypeChain files into lib folder
cp -R ./src/tsc/web3 ./lib/tsc/web3;

# Copy Ganache workaround
cp ./src/ganache-core.d.ts ./lib;

# Copy smart contract addresses
cp -R src/ethereum/addresses.* ./lib/ethereum