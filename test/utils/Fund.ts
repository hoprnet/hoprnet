import BigNumber from "bignumber.js";
import { keccak256, signMessage } from "./random";

BigNumber.config({ EXPONENTIAL_AT: 1e9 });

type IFund = (args: {
  web3: any;
  state_counter: string;
  initiator: string;
  deposit: string;
  party_a_amount: string;
  not_after: string;
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
  state_counter,
  initiator,
  deposit,
  party_a_amount,
  not_after,
  signerPrivKey
}) => {
  const hashedFund = keccak256(
    { type: "uint256", value: state_counter },
    { type: "address", value: initiator },
    { type: "uint256", value: deposit },
    { type: "uint256", value: party_a_amount },
    { type: "uint256", value: not_after }
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
