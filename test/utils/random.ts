import BigNumber from "bignumber.js";
const BN = require("bn.js");
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

// inputs should be a bytes32 string e.g: "0x..."
export const xorBytes32 = (a: string, b: string) => {
  return `0x${new BN(a.slice(2), 16).xor(new BN(b.slice(2), 16)).toString(16)}`;
};

export const isPartyA = (a: string, b: string) => {
  return new BN(a.slice(2), 16).lt(new BN(b.slice(2), 16))
}

export const getParties = (a: string, b: string) => {
  if (isPartyA(a, b)) {
    return {
      party_a: a,
      party_b: b
    }
  }

  return {
    party_a: b,
    party_b: a
  }
}

export const getChannelId = (party_a: string, party_b: string) => {
  return keccak256({
    type: "address",
    value: party_a
  },{
    type: "address",
    value: party_b
  })
}