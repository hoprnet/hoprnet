const Web3 = require("web3");

export const createMessage = (contract: string, amount: string): string => {
  return Web3.utils.soliditySha3(
    { type: "address", value: contract },
    { type: "uint256", value: amount }
  );
};

export const prefixMessageHash = (messageHash: string) => {
  return Web3.utils.soliditySha3(
    {
      type: "string",
      value: "\x19Ethereum Signed Message:\n32"
    },
    {
      type: "bytes32",
      value: messageHash
    }
  );
};

export const signMessage = (
  web3: any,
  message: string,
  senderPrivKey: string
) => {
  return web3.eth.accounts.sign(message, senderPrivKey);
};

export const signPayment = (
  web3: any,
  senderPrivKey: string,
  contract: string,
  amount: string
) => {
  const messageHash = createMessage(contract, amount);

  return signMessage(web3, messageHash, senderPrivKey);
};

export const recoverSigner = (
  web3: any,
  message: string,
  signature: string
) => {
  return web3.eth.accounts.recover(message, signature, false);
};
