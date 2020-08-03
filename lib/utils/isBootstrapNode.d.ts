import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '@hoprnet/hopr-core';
import type PeerId from 'peer-id';
/**
 * Checks whether the given PeerId belongs to any known bootstrap node.
 *
 * @param peerId
 */
export declare function isBootstrapNode(node: Hopr<HoprCoreConnector>, peerId: PeerId): boolean;
