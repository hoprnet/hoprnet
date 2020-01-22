import BigNumber from "bignumber.js";
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

const MAX_UINT256 = new BigNumber(2).pow(256).minus(1);

export const createTicket = (ops: {
  web3: any;
  senderPrivKey: string;
  secret: string;
  amount: string;
  winProbPercent: string;
}) => {
  const { web3, senderPrivKey, secret, amount, winProbPercent } = ops;

  const s_a = Web3.utils.stringToHex("s_a");
  const s_b = Web3.utils.stringToHex("s_b");
  const hashed_s_a = Web3.utils.keccak256(s_a);
  const hashed_s_b = Web3.utils.keccak256(s_b);
  const challenge = Web3.utils.keccak256(hashed_s_a, hashed_s_b);

  const hashedSecret = Web3.utils.keccak256(secret);

  BigNumber.config({ EXPONENTIAL_AT: 1e9 });
  const winProb = web3.utils.fromAscii(
    new BigNumber(winProbPercent)
      .multipliedBy(MAX_UINT256)
      .dividedBy(100)
      .toString()
  );

  const signature = signMessage(
    web3,
    Web3.utils.keccak256(challenge, hashedSecret, amount, winProb),
    senderPrivKey
  );

  return {
    s_a,
    s_b,
    challenge,
    hashedSecret,
    amount,
    winProb,
    signature
  };
};
