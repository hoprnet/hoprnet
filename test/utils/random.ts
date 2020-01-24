import BigNumber from "bignumber.js";
const Web3 = require("web3");

export const MAX_UINT256 = new BigNumber(2).pow(256).minus(1);

export const keccak256 = (
  ...args: { type: string; value: string | number }[]
): string => {
  return Web3.utils.soliditySha3(...args);
};

export const signMessage = (
  web3: any,
  message: string,
  senderPrivKey: string
) => {
  return web3.eth.accounts.sign(message, senderPrivKey);
};

export const recoverSigner = (
  web3: any,
  message: string,
  signature: string
) => {
  return web3.eth.accounts.recover(message, signature, false);
};
