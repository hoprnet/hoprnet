import PeerInfo from 'peer-info';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '..';
import { CrawlResponse } from '../messages';
declare class Crawler<Chain extends HoprCoreConnector> {
    node: Hopr<Chain>;
    constructor(node: Hopr<Chain>);
    crawl(comparator?: (peerInfo: PeerInfo) => boolean): Promise<void>;
    handleCrawlRequest(): Generator<CrawlResponse, void, unknown>;
    printStatsAndErrors(contactedPeerIds: Set<string>, errors: Error[], now: number, before: number): void;
}
export { Crawler };
