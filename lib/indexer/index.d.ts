import type { Indexer as IIndexer } from '@hoprnet/hopr-core-connector-interface';
import type HoprEthereum from '..';
import BN from 'bn.js';
import { ChannelEntry, Public } from '../types';
import { ContractEventLog } from '../tsc/web3/types';
declare type LightEvent<E extends ContractEventLog<any>> = Pick<E, 'event' | 'blockNumber' | 'transactionHash' | 'transactionIndex' | 'logIndex' | 'returnValues'>;
declare type Channel = {
    partyA: Public;
    partyB: Public;
    channelEntry: ChannelEntry;
};
export declare type OpenedChannelEvent = LightEvent<ContractEventLog<{
    opener: Public;
    counterparty: Public;
}>>;
export declare type ClosedChannelEvent = LightEvent<ContractEventLog<{
    closer: Public;
    counterparty: Public;
    partyAAmount?: BN;
    partyBAmount?: BN;
}>>;
/**
 * Simple indexer to keep track of all open payment channels.
 */
declare class Indexer implements IIndexer {
    private connector;
    private log;
    private status;
    private unconfirmedEvents;
    private starting?;
    private stopping?;
    private newBlockEvent?;
    private openedChannelEvent?;
    private closedChannelEvent?;
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
    has(partyA: Public, partyB: Public): Promise<boolean>;
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
        partyA?: Public;
        partyB?: Public;
    }): Promise<Channel[]>;
    private store;
    private delete;
    private compareUnconfirmedEvents;
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
    private processOpenedChannelEvent;
    private processClosedChannelEvent;
}
export default Indexer;
