import type { Channel as IChannel } from '@hoprnet/hopr-core-connector-interface';
import { AccountId, Balance, ChannelBalance, Channel as ChannelType, Hash, Moment, Signature, SignedChannel, SignedTicket } from '../types';
import TicketFactory from './ticket';
import type HoprEthereum from '..';
declare class Channel implements IChannel {
    coreConnector: HoprEthereum;
    counterparty: Uint8Array;
    private _signedChannel;
    private _settlementWindow?;
    private _channelId?;
    ticket: TicketFactory;
    constructor(coreConnector: HoprEthereum, counterparty: Uint8Array, signedChannel: SignedChannel);
    private onceClosed;
    private onClose;
    private get channel();
    private get status();
    get offChainCounterparty(): Promise<Uint8Array>;
    get channelId(): Promise<Hash>;
    get settlementWindow(): Promise<Moment>;
    get state(): Promise<ChannelType>;
    get balance(): Promise<Balance>;
    get balance_a(): Promise<Balance>;
    get currentBalance(): Promise<Balance>;
    get currentBalanceOfCounterparty(): Promise<Balance>;
    initiateSettlement(): Promise<void>;
    getPreviousChallenges(): Promise<Hash>;
    testAndSetNonce(signature: Uint8Array): Promise<void>;
}
declare class ChannelFactory {
    private coreConnector;
    constructor(coreConnector: HoprEthereum);
    increaseFunds(counterparty: AccountId, amount: Balance): Promise<void>;
    isOpen(counterpartyPubKey: Uint8Array): Promise<boolean>;
    createDummyChannelTicket(counterParty: AccountId, challenge: Hash, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Promise<SignedTicket>;
    createSignedChannel(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        channel: ChannelType;
        signature?: Signature;
    }): Promise<SignedChannel>;
    create(counterpartyPubKey: Uint8Array, _getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>, channelBalance?: ChannelBalance, sign?: (channelBalance: ChannelBalance) => Promise<SignedChannel>): Promise<Channel>;
    getAll<T, R>(onData: (channel: Channel) => Promise<T>, onEnd: (promises: Promise<T>[]) => R): Promise<R>;
    closeChannels(): Promise<Balance>;
    handleOpeningRequest(): (source: AsyncIterable<Uint8Array>) => AsyncIterable<Uint8Array>;
}
export { ChannelFactory };
export default Channel;
