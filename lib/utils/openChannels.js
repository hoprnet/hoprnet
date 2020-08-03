"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getOpenChannels = exports.getPartyOpenChannels = exports.getMyOpenChannels = exports.getPeers = void 0;
const peer_id_1 = __importDefault(require("peer-id"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const utils_1 = require("@hoprnet/hopr-core/lib/utils");
const isBootstrapNode_1 = require("./isBootstrapNode");
/**
 * Get node's peers.
 * @returns an array of peer ids
 */
function getPeers(node, ops = {
    noBootstrapNodes: false,
}) {
    let peers = node.network.peerStore.peers.map((peer) => peer_id_1.default.createFromB58String(peer.id));
    if (ops.noBootstrapNodes) {
        peers = peers.filter((peerId) => {
            return !isBootstrapNode_1.isBootstrapNode(node, peerId);
        });
    }
    return peers;
}
exports.getPeers = getPeers;
/**
 * Get node's open channels by looking into connector's DB.
 * @returns a promise that resolves to an array of peer ids
 */
function getMyOpenChannels(node) {
    return new Promise((resolve, reject) => {
        try {
            let peerIds = [];
            node.paymentChannels.channel.getAll(async (channel) => {
                const pubKey = await channel.offChainCounterparty;
                const peerId = await utils_1.pubKeyToPeerId(pubKey);
                if (!peerIds.includes(peerId)) {
                    peerIds.push(peerId);
                }
            }, async (promises) => {
                await Promise.all(promises);
                return resolve(peerIds);
            });
        }
        catch (err) {
            return reject(err);
        }
    });
}
exports.getMyOpenChannels = getMyOpenChannels;
/**
 * Get node's open channels and a counterParty's using connector's indexer.
 * @returns a promise that resolves to an array of peer ids
 */
async function getPartyOpenChannels(node, party) {
    const { indexer, utils } = node.paymentChannels;
    const partyAccountId = await utils.pubKeyToAccountId(party.pubKey.marshal());
    // get indexed open channels
    const channels = await indexer.get({
        partyA: partyAccountId,
    });
    // get the counterparty of each channel
    const channelAccountIds = channels.map((channel) => {
        return hopr_utils_1.u8aEquals(channel.partyA, partyAccountId) ? channel.partyB : channel.partyA;
    });
    // get available nodes
    const peers = await Promise.all(getPeers(node, {
        noBootstrapNodes: true,
    }).map(async (peer) => {
        return {
            peer,
            accountId: await utils.pubKeyToAccountId(peer.pubKey.marshal()),
        };
    }));
    return peers.reduce((acc, { peer, accountId }) => {
        if (channelAccountIds.find((channelAccountId) => {
            return hopr_utils_1.u8aEquals(accountId, channelAccountId);
        })) {
            acc.push(peer);
        }
        return acc;
    }, []);
}
exports.getPartyOpenChannels = getPartyOpenChannels;
/**
 * Get node's open channels with a counterParty using connector's DB or indexer if supported.
 * @returns a promise that resolves to an array of peer ids
 */
async function getOpenChannels(node, partyPeerId) {
    const supportsIndexer = typeof node.paymentChannels.indexer !== 'undefined';
    const partyIfSelf = node.peerInfo.id.equals(partyPeerId);
    if (partyIfSelf) {
        // if party is self, and indexer not supported
        return getMyOpenChannels(node);
    }
    else if (supportsIndexer) {
        // if connector supports indexeer
        return getPartyOpenChannels(node, partyPeerId);
    }
    else {
        // return an emptry array if connector does not support indexer
        return [];
    }
}
exports.getOpenChannels = getOpenChannels;
//# sourceMappingURL=openChannels.js.map