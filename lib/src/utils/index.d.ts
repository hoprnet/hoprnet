export * from "@hoprnet/hopr-utils";
export * from './crypto';
export * from './fs';
export * from './libp2p';
export * from './persistence';
export * from './concurrency';
/**
 * Mine a single block and increase the timestamp by the given amount.
 *
 * @notice The purpose of this method is to use it for testing with a local
 * testnet, i. e. Ganache.
 *
 * @param {Object} provider a valid Web3 provider
 * @param {Number} amountOfTime increase the timestamp by that amount of time, default 1 minute
 */
/**
 * Checks whether one of the src files is newer than one of
 * the artifacts.
 *
 * @notice the method utilizes Truffle to compile the smart contracts.
 * Please make sure that Truffle is accessible by `npx`.
 *
 * @param {Array} srcFiles the absolute paths of the source files
 * @param {Array} artifacts the absolute paths of the artifacts
 */
/**
 * Deploys the smart contract.
 *
 * @param index current index of the account of `FUNDING_PEER`
 * @param web3 instance of web3.js
 * @returns promise that resolve once the contract is compiled and deployed, otherwise
 * it rejects.
 */
/**
 * Takes a contract address and changes every occurence of `CONTRACT_ADDRESS = //...` to
 * the given contract address
 * @param fileNames the files whose CONTRACT_ADDRESS should be changed
 * @param contractAddress the new contract address
 */
