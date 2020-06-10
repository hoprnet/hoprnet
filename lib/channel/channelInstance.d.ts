import type { Channel as IChannel } from '@hoprnet/hopr-core-connector-interface';
import { Balance, Channel as ChannelType, Hash, Moment, SignedChannel } from '../types';
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
export default Channel;
