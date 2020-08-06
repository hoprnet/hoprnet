import type HoprEthereum from '.';
import { AccountId, Balance, Hash, NativeBalance, TicketEpoch } from './types';
declare class Account {
    coreConnector: HoprEthereum;
    private _address?;
    private _nonceIterator;
    private _ticketEpoch;
    private _ticketEpochListener;
    /**
     * The accounts keys:
     */
    keys: {
        onChain: {
            privKey: Uint8Array;
            pubKey: Uint8Array;
        };
        offChain: {
            privKey: Uint8Array;
            pubKey: Uint8Array;
        };
    };
    constructor(coreConnector: HoprEthereum, privKey: Uint8Array, pubKey: Uint8Array);
    get nonce(): Promise<number>;
    /**
     * Returns the current balances of the account associated with this node (HOPR)
     * @returns a promise resolved to Balance
     */
    get balance(): Promise<Balance>;
    /**
     * Returns the current native balance (ETH)
     * @returns a promise resolved to Balance
     */
    get nativeBalance(): Promise<NativeBalance>;
    get ticketEpoch(): Promise<TicketEpoch>;
    /**
     * Returns the current value of the onChainSecret
     */
    get onChainSecret(): Promise<Hash>;
    get address(): Promise<AccountId>;
}
export default Account;
