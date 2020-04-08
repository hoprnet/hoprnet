import { CrawlStatus } from '.';
import PeerInfo from 'peer-info';
declare class CrawlResponse extends Uint8Array {
    constructor(arr?: Uint8Array, struct?: {
        status: CrawlStatus;
        peerInfos?: PeerInfo[];
    });
    subarray(begin?: number, end?: number): Uint8Array;
    get statusRaw(): Uint8Array;
    get status(): CrawlStatus;
    get peerInfosRaw(): Uint8Array;
    get peerInfos(): Promise<PeerInfo[]>;
}
export { CrawlResponse };
