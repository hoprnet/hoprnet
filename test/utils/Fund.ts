import { keccak256, signMessage } from "./random";

type IFund = (args: {
  web3: any;
  stateCounter: string;
  initiator: string;
  deposit: string;
  partyAAmount: string;
  notAfter: string;
  signerPrivKey: string;
}) => {
  hashedFund: string; // return hashed alternative
  signature: string; // signature of hashedTicket
  r: string;
  s: string;
  v: string;
};

/*
  prepares fund payload
*/
const Fund: IFund = ({
  web3,
  stateCounter,
  initiator,
  deposit,
  partyAAmount,
  notAfter,
  signerPrivKey
}) => {
  const hashedFund = keccak256(
    { type: "uint256", value: stateCounter },
    { type: "address", value: initiator },
    { type: "uint256", value: deposit },
    { type: "uint256", value: partyAAmount },
    { type: "uint256", value: notAfter }
  );

  const { signature, r, s, v } = signMessage(web3, hashedFund, signerPrivKey);

  return {
    hashedFund,
    signature,
    r,
    s,
    v
  };
};

export { Fund };
