import BigNumber from "bignumber.js";
import { keccak256, xorBytes32, signMessage, MAX_UINT256 } from "./random";

BigNumber.config({ EXPONENTIAL_AT: 1e9 });

type ITicket = (args: {
  web3: any;
  sender: string;
  recipient: string;
  senderPrivKey: string;
  porSecretA: string; // needs to be bytes32
  porSecretB: string; // needs to be bytes32
  recipientSecret: string; // needs to be bytes32
  amount: string;
  counter: number;
  winProbPercent: string; // max 100
}) => {
  sender: string; // return same as provided
  recipient: string; // return same as provided
  porSecretA: string; // return same as provided
  porSecretB: string; // return same as provided
  recipientSecret: string; // return same as provided
  amount: string; // return same as provided
  counter: number; // return same as provided
  hashedPorSecretA: string; // return hashed alternative
  hashedPorSecretB: string; // return hashed alternative
  challenge: string; // return hashed alternative
  hashedRecipientSecret: string; // return hashed alternative
  winProb: string; // return winProb in bytes32
  hashedTicket: string; // return hashed alternative
  signature: string; // signature of hashedTicket
};

/*
  prepares ticket payload
*/
const Ticket: ITicket = ({
  web3,
  sender,
  recipient,
  senderPrivKey,
  porSecretA,
  porSecretB,
  recipientSecret,
  amount,
  counter,
  winProbPercent
}) => {
  // proof of relay related hashes
  const hashedPorSecretA = keccak256({ type: "bytes32", value: porSecretA });
  const hashedPorSecretB = keccak256({ type: "bytes32", value: porSecretB });
  const challenge = xorBytes32(hashedPorSecretA, hashedPorSecretB);

  // proof of randomness related hashes
  const hashedRecipientSecret = keccak256({
    type: "bytes32",
    value: recipientSecret
  });

  // calculate win probability in bytes32
  const winProb = web3.utils.numberToHex(
    new BigNumber(winProbPercent)
      .multipliedBy(MAX_UINT256)
      .dividedBy(100)
      .toString()
  );

  const hashedTicket = keccak256(
    { type: "bytes32", value: challenge },
    { type: "bytes32", value: hashedRecipientSecret },
    { type: "uint256", value: counter },
    { type: "uint256", value: amount },
    { type: "bytes32", value: winProb }
  );

  // sender signs ticket
  const { signature } = signMessage(web3, hashedTicket, senderPrivKey);

  return {
    sender,
    recipient,
    porSecretA,
    porSecretB,
    recipientSecret,
    amount,
    counter,
    hashedPorSecretA,
    hashedPorSecretB,
    challenge,
    hashedRecipientSecret,
    winProb,
    hashedTicket,
    signature
  };
};

export { Ticket };
