import type { Channel as IChannel, Types } from '@hoprnet/hopr-core-connector-interface';
import { AccountId, Balance, ChannelBalance, Hash, Moment, SignedChannel, SignedTicket, State } from '../types';
import type HoprEthereum from '..';
declare class Channel implements IChannel {
    coreConnector: HoprEthereum;
    counterparty: Uint8Array;
    private _signedChannel;
    private _settlementWindow?;
    private _channelId?;
    ticket: typeof Types.Ticket;
    constructor(coreConnector: HoprEthereum, counterparty: Uint8Array, signedChannel: SignedChannel);
    private onceClosed;
    private onClose;
    private get channel();
    private get status();
    get offChainCounterparty(): Promise<Uint8Array>;
    get channelId(): Promise<Hash>;
    get settlementWindow(): Promise<Moment>;
    get state(): Promise<State>;
    get balance(): Promise<Balance>;
    get balance_a(): Promise<Balance>;
    get currentBalance(): Promise<Balance>;
    get currentBalanceOfCounterparty(): Promise<Balance>;
    initiateSettlement(): Promise<void>;
    getPreviousChallenges(): Promise<Hash>;
    testAndSetNonce(signature: Uint8Array): Promise<void>;
    static isOpen(this: HoprEthereum, counterpartyPubKey: Uint8Array): Promise<boolean>;
    static increaseFunds(this: HoprEthereum, counterparty: AccountId, amount: Balance): Promise<void>;
    static createDummyChannelTicket(this: HoprEthereum, counterParty: AccountId, challenge: Hash, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Promise<SignedTicket>;
    static create(this: HoprEthereum, counterpartyPubKey: Uint8Array, _getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>, channelBalance?: ChannelBalance, sign?: (channelBalance: ChannelBalance) => Promise<SignedChannel>): Promise<Channel>;
    static getAll<T, R>(this: HoprEthereum, onData: (channel: Channel) => Promise<T>, onEnd: (promises: Promise<T>[]) => R): Promise<R>;
    static closeChannels(this: HoprEthereum): Promise<Balance>;
    static handleOpeningRequest(this: HoprEthereum): (source: AsyncIterable<Uint8Array>) => AsyncIterable<Uint8Array>;
}
export default Channel;
