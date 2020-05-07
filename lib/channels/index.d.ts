import type HoprEthereum from '..';
import { BlockHeader } from 'web3-eth';
import { AccountId, ChannelEntry } from '../types';
import { ContractEventLog } from '../tsc/web3/types';
declare type Channel = {
    partyA: AccountId;
    partyB: AccountId;
    channelEntry: ChannelEntry;
};
declare type OpenedChannelEvent = ContractEventLog<{
    opener: string;
    counterParty: string;
}>;
declare type ClosedChannelEvent = ContractEventLog<{
    closer: string;
    counterParty: string;
}>;
/**
 * Barebones indexer to keep track of all open payment channels.
 * Eventually we will move to a better solution.
 */
declare class Channels {
    static getLatestConfirmedBlockNumber(connector: HoprEthereum): Promise<number>;
    static has(connector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<boolean>;
    static get(connector: HoprEthereum, query?: {
        partyA?: AccountId;
        partyB?: AccountId;
    }): Promise<Channel[]>;
    static getAll(connector: HoprEthereum): Promise<Channel[]>;
    static store(connector: HoprEthereum, partyA: AccountId, partyB: AccountId, channelEntry: ChannelEntry): Promise<void>;
    static delete(connector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<void>;
    static onNewBlock(connector: HoprEthereum, block: BlockHeader): Promise<void>;
    static onOpenedChannel(connector: HoprEthereum, event: OpenedChannelEvent): Promise<void>;
    static onClosedChannel(connector: HoprEthereum, event: ClosedChannelEvent): Promise<void>;
    static start(connector: HoprEthereum): Promise<boolean>;
    static stop(): Promise<boolean>;
}
export default Channels;
