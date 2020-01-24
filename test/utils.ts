import BigNumber from "bignumber.js";
const Web3 = require("web3");

export const MAX_UINT256 = new BigNumber(2).pow(256).minus(1);

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

// TODO: improve this
export const createTicket = (ops: {
  web3: any;
  senderPrivKey: string;
  secret: string;
  counter: number;
  amount: string;
  winProbPercent: string;
}) => {
  const { web3, senderPrivKey, secret, counter, amount, winProbPercent } = ops;

  const s_a = Web3.utils.keccak256("s_a");
  const s_b = Web3.utils.keccak256("s_b");
  const hashed_s_a = Web3.utils.keccak256(s_a);
  const hashed_s_b = Web3.utils.keccak256(s_b);
  const challenge = Web3.utils.soliditySha3(
    {
      type: "bytes32",
      value: hashed_s_a
    },
    {
      type: "bytes32",
      value: hashed_s_b
    }
  );
  const hashedSecret = Web3.utils.keccak256(secret);

  BigNumber.config({ EXPONENTIAL_AT: 1e9 });
  const winProbInt = new BigNumber(winProbPercent)
    .multipliedBy(MAX_UINT256)
    .dividedBy(100)
    .toString();

  const winProb = web3.utils.numberToHex(winProbInt);

  const signature = signMessage(
    web3,
    Web3.utils.soliditySha3(
      { type: "bytes32", value: challenge },
      { type: "bytes32", value: hashedSecret },
      { type: "uint256", value: counter },
      { type: "uint256", value: amount },
      { type: "bytes32", value: winProb }
    ),
    senderPrivKey
  );

  return {
    s_a,
    s_b,
    hashed_s_a,
    hashed_s_b,
    challenge,
    hashedSecret,
    counter,
    amount,
    winProb,
    signature
  };
};
