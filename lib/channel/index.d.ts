import { AccountId, Balance, ChannelBalance, Channel as ChannelType, Hash, Public, Signature, SignedChannel, SignedTicket } from '../types';
import type HoprEthereum from '..';
import Channel from './channel';
import { OnChainChannel } from './types';
declare class ChannelFactory {
    private coreConnector;
    constructor(coreConnector: HoprEthereum);
    increaseFunds(counterparty: AccountId, amount: Balance): Promise<void>;
    isOpen(counterpartyPubKey: Uint8Array): Promise<boolean>;
    createDummyChannelTicket(counterparty: AccountId, challenge: Hash, arr?: {
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
    handleOpeningRequest(source: AsyncIterable<Uint8Array>): any;
    getOffChainState(counterparty: Uint8Array): Promise<SignedChannel>;
    saveOffChainState(counterparty: Uint8Array, signedChannel: SignedChannel): Promise<void>;
    deleteOffChainState(counterparty: Uint8Array): Promise<void>;
    getOnChainState(channelId: Hash): Promise<OnChainChannel>;
    onceOpen(self: Public, counterparty: Public): Promise<unknown>;
    onceClosed(self: Public, counterparty: Public): Promise<unknown>;
}
export { ChannelFactory };
export default Channel;
