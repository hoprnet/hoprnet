import PeerInfo from 'peer-info';
/**
 * Serializes peerInfos including their multiaddrs.
 * @param peerInfo PeerInfo to serialize
 */
declare function serializePeerInfo(peerInfo: PeerInfo): Uint8Array;
export { serializePeerInfo };
