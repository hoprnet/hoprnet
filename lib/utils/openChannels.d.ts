import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '@hoprnet/hopr-core';
import PeerId from 'peer-id';
/**
 * Get node's peers.
 * @returns an array of peer ids
 */
export declare function getPeers(node: Hopr<HoprCoreConnector>, ops?: {
    noBootstrapNodes: boolean;
}): PeerId[];
/**
 * Get node's open channels by looking into connector's DB.
 * @returns a promise that resolves to an array of peer ids
 */
export declare function getMyOpenChannels(node: Hopr<HoprCoreConnector>): Promise<PeerId[]>;
/**
 * Get node's open channels and a counterParty's using connector's indexer.
 * @returns a promise that resolves to an array of peer ids
 */
export declare function getPartyOpenChannels(node: Hopr<HoprCoreConnector>, party: PeerId): Promise<PeerId[]>;
/**
 * Get node's open channels with a counterParty using connector's DB or indexer if supported.
 * @returns a promise that resolves to an array of peer ids
 */
export declare function getOpenChannels(node: Hopr<HoprCoreConnector>, partyPeerId: PeerId): Promise<PeerId[]>;
