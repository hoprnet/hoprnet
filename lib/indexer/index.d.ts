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
     * Get all stored channel entries, if party is provided,
     * it will return the open channels of the given party.
     *
     * @returns promise that resolves to a list of channel entries
     */
    private getAll;
    /**
     * Get stored channel of the given parties.
     *
     * @returns promise that resolves to a channel entry or undefined if not found
     */
    private getSingle;
    /**
     * Get stored channels entries.
     *
     * If query is left empty, it will return all channels.
     *
     * If only one party is provided, it will return all channels of the given party.
     *
     * If both parties are provided, it will return the channel of the given party.
     *
     * @param query
     * @returns promise that resolves to a list of channel entries
     */
    get(query?: {
        partyA?: AccountId;
        partyB?: AccountId;
    }): Promise<Channel[]>;
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
