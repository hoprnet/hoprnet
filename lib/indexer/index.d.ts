import type { Indexer as IIndexer } from '@hoprnet/hopr-core-connector-interface';
import type HoprEthereum from '..';
import { AccountId, ChannelEntry } from '../types';
declare type Channel = {
    partyA: AccountId;
    partyB: AccountId;
    channelEntry: ChannelEntry;
};
/**
 * Simple indexer to keep track of all open payment channels.
 */
declare class Indexer implements IIndexer {
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
    /**
     * Returns the latest confirmed block number.
     *
     * @returns promive that resolves to a number
     */
    private getLatestConfirmedBlockNumber;
    /**
     * Check if channel entry exists.
     *
     * @returns promise that resolves to true or false
     */
    has(partyA: AccountId, partyB: AccountId): Promise<boolean>;
    /**
     * Get stored channels entries.
     *
     * @param query
     * @returns promise that resolves to a list of channel entries
     */
    get(query?: {
        partyA?: AccountId;
        partyB?: AccountId;
    }): Promise<Channel[]>;
    /**
     * Get all stored channels entries.
     *
     * @returns promise that resolves to a list of channel entries
     */
    getAll(): Promise<Channel[]>;
    private store;
    private delete;
    private onNewBlock;
    private onOpenedChannel;
    private onClosedChannel;
    /**
     * Start indexing,
     * listen to all open / close events,
     * store entries after X confirmations.
     *
     * @returns true if start was succesful
     */
    start(): Promise<boolean>;
    /**
     * Stop indexing.
     *
     * @returns true if stop was succesful
     */
    stop(): Promise<boolean>;
}
export default Indexer;
