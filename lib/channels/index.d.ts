import type HoprEthereum from '..';
import { BlockHeader } from 'web3-eth';
import { AccountId } from '../types';
import { ContractEventLog } from '../tsc/web3/types';
declare type Channel = {
    partyA: AccountId;
    partyB: AccountId;
    blockNumber: number;
};
declare type OpenedChannelEvent = ContractEventLog<{
    opener: string;
    counterParty: string;
}>;
declare type ClosedChannelEvent = ContractEventLog<{
    closer: string;
    counterParty: string;
}>;
declare class Channels {
    static getLatestConfirmedBlockNumber(coreConnector: HoprEthereum): Promise<number>;
    static has(coreConnector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<boolean>;
    static get(coreConnector: HoprEthereum, query?: {
        partyA?: AccountId;
        partyB?: AccountId;
    }): Promise<Channel[]>;
    static getAll(coreConnector: HoprEthereum): Promise<Channel[]>;
    static store(coreConnector: HoprEthereum, partyA: AccountId, partyB: AccountId, blockNumber: number): Promise<[void, void]>;
    static delete(coreConnector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<void>;
    static onNewBlock(coreConnector: HoprEthereum, block: BlockHeader): Promise<void>;
    static onOpenedChannel(coreConnector: HoprEthereum, event: OpenedChannelEvent): Promise<void>;
    static onClosedChannel(coreConnector: HoprEthereum, event: ClosedChannelEvent): Promise<void>;
    static start(coreConnector: HoprEthereum): Promise<boolean>;
    static stop(): Promise<boolean>;
}
export default Channels;
