echo $PWD
ls ../ethereum

[ ! -d "./src/ethereum" ] && mkdir ./src/ethereum
[ ! -d "./src/ethereum/abi" ] && mkdir ./src/ethereum/abi


# Make the smart contract ABI available to our Typescript sources
cp -R ../ethereum/build/extracted/abis/*.json ./src/ethereum/abi;

# Make our smart contract addresses available to our Typescript sources
cp ../ethereum/build/lib/scripts/addresses.* ./src/ethereum

# Compile our Typescript sources
yarn run tsc;

# Copy generated TypeChain files into lib folder
cp -R ./src/tsc/web3 ./lib/tsc/web3;

# Copy smart contract addresses
cp -R src/ethereum/addresses.* ./lib/ethereum
