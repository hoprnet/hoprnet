import type HoprEthereum from '..';
import { AccountId } from '../types';
declare type Channel = {
    partyA: AccountId;
    partyB: AccountId;
    blockNumber: number;
};
declare class Channels {
    static has(coreConnector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<boolean>;
    static get(coreConnector: HoprEthereum, query?: {
        partyA?: AccountId;
        partyB?: AccountId;
    }): Promise<Channel[]>;
    static getAll(coreConnector: HoprEthereum): Promise<Channel[]>;
    static store(coreConnector: HoprEthereum, partyA: AccountId, partyB: AccountId, blockNumber: number): Promise<[void, void]>;
    static delete(coreConnector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<void>;
    static start(coreConnector: HoprEthereum): Promise<void>;
}
export default Channels;
