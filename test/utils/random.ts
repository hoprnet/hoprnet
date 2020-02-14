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
  signerPrivKey: string
) => {
  return web3.eth.accounts.sign(message, signerPrivKey);
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
  return new BN(a.slice(2), 16).lt(new BN(b.slice(2), 16));
};

export const getParties = (a: string, b: string) => {
  if (isPartyA(a, b)) {
    return {
      partyA: a,
      partyB: b
    };
  }

  return {
    partyA: b,
    partyB: a
  };
};

export const getChannelId = (partyA: string, partyB: string) => {
  return keccak256(
    {
      type: "address",
      value: partyA
    },
    {
      type: "address",
      value: partyB
    }
  );
};

// export const generatePorSecrets = (preImage: string, iterations: number) => {
//   let result = [preImage];

//   while (iterations-- > 0) {
//     result.unshift(
//       keccak256({
//         type: "bytes32",
//         value: keccak256({
//           type: "bytes32",
//           value: result[result.length - 1]
//         })
//       })
//     );
//   }

//   return result;
// };
