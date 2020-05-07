import type HoprEthereum from '..';
import { AccountId, ChannelEntry } from '../types';
declare type Channel = {
    partyA: AccountId;
    partyB: AccountId;
    channelEntry: ChannelEntry;
};
/**
 * Barebones indexer to keep track of all open payment channels.
 * Eventually we will move to a better solution.
 */
declare class Channels {
    private connector;
    private log;
    private status;
    private unconfirmedEvents;
    private starting;
    private stopping;
    private newBlockEvent;
    private openedChannelEvent;
    private closedChannelEvent;
    constructor(connector: HoprEthereum);
    private getLatestConfirmedBlockNumber;
    has(partyA: AccountId, partyB: AccountId): Promise<boolean>;
    get(query?: {
        partyA?: AccountId;
        partyB?: AccountId;
    }): Promise<Channel[]>;
    getAll(): Promise<Channel[]>;
    private store;
    private delete;
    private onNewBlock;
    private onOpenedChannel;
    private onClosedChannel;
    start(): Promise<boolean>;
    stop(): Promise<boolean>;
}
export default Channels;
