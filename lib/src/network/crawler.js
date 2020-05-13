"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Crawler = void 0;
const chalk_1 = __importDefault(require("chalk"));
const utils_1 = require("../utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const constants_1 = require("../constants");
const messages_1 = require("../messages");
const MAX_PARALLEL_REQUESTS = 4;
class Crawler {
    constructor(node) {
        this.node = node;
    }
    async crawl(comparator = () => true) {
        const errors = [];
        // fast non-inclusion check
        const contactedPeerIds = new Set(); // @TODO could be replaced by a bloom filter
        // enumerable
        const unContactedPeerIdArray = []; // @TODO add new peerIds lazily
        // fast non-inclusion check
        const unContactedPeerIdSet = new Set(); // @TODO replace this by a sorted array
        let before = 0; // store current state
        for (const peerInfo of this.node.peerStore.peers.values()) {
            unContactedPeerIdArray.push(peerInfo);
            if (comparator(peerInfo)) {
                before += 1;
            }
        }
        const tokens = utils_1.getTokens(MAX_PARALLEL_REQUESTS);
        /**
         * Get all known nodes that match our requirements.
         */
        const getCurrentNodes = () => {
            let currentNodes = 0;
            for (const peerInfo of this.node.peerStore.peers.values()) {
                if (comparator(peerInfo) == true) {
                    currentNodes += 1;
                }
            }
            return currentNodes;
        };
        /**
         * Check if we're finished
         */
        const isDone = () => {
            return contactedPeerIds.size >= constants_1.MAX_HOPS && getCurrentNodes() >= constants_1.MAX_HOPS;
        };
        /**
         * Returns a random node and removes it from the array.
         */
        const getRandomNode = () => {
            if (unContactedPeerIdArray.length == 0) {
                throw Error(`Cannot pick a random node because there are none.`);
            }
            const index = hopr_utils_1.randomInteger(0, unContactedPeerIdArray.length);
            if (index == unContactedPeerIdArray.length - 1) {
                return unContactedPeerIdArray.pop();
            }
            const selected = unContactedPeerIdArray[index];
            unContactedPeerIdArray[index] = unContactedPeerIdArray.pop();
            return selected;
        };
        /**
         * Stores the crawling "threads"
         */
        const promises = [];
        /**
         * Connect to another peer and returns a promise that resolves to all received nodes
         * that were previously unknown.
         *
         * @param peerInfo PeerInfo of the peer that is queried
         */
        const queryNode = async (peerInfo, token) => {
            let peerInfos;
            if (isDone()) {
                tokens.push(token);
                return;
            }
            // Start additional "threads"
            while (tokens.length > 0 && unContactedPeerIdArray.length > 0) {
                const token = tokens.pop();
                const currentNode = getRandomNode();
                if (promises[token] != null) {
                    /**
                     * @TODO remove this and make sure that the Promise is always
                     * already resolved.
                     */
                    await promises[token];
                    promises[token] = queryNode(currentNode, token);
                }
                else {
                    promises.push(queryNode(currentNode, token));
                }
            }
            unContactedPeerIdSet.delete(peerInfo.id.toB58String());
            contactedPeerIds.add(peerInfo.id.toB58String());
            try {
                peerInfos = await this.node.interactions.network.crawler.interact(peerInfo);
            }
            catch (err) {
                errors.push(err);
            }
            finally {
                if (peerInfos != null && Array.isArray(peerInfos)) {
                    for (let i = 0; i < peerInfos.length; i++) {
                        if (peerInfos[i].id.isEqual(this.node.peerInfo.id)) {
                            continue;
                        }
                        if (!contactedPeerIds.has(peerInfos[i].id.toB58String()) && !unContactedPeerIdSet.has(peerInfos[i].id.toB58String())) {
                            unContactedPeerIdSet.add(peerInfos[i].id.toB58String());
                            unContactedPeerIdArray.push(peerInfos[i]);
                            this.node.peerStore.put(peerInfos[i]);
                        }
                    }
                }
            }
            if (unContactedPeerIdArray.length > 0) {
                return queryNode(getRandomNode(), token);
            }
            tokens.push(token);
            return;
        };
        for (let i = 0; i < MAX_PARALLEL_REQUESTS && unContactedPeerIdArray.length > 0; i++) {
            promises.push(queryNode(getRandomNode(), tokens.pop()));
        }
        if (!isDone()) {
            await Promise.all(promises);
        }
        this.printStatsAndErrors(contactedPeerIds, errors, getCurrentNodes(), before);
        if (!isDone()) {
            throw Error(`Unable to find enough other nodes in the network.`);
        }
        unContactedPeerIdSet.clear();
        contactedPeerIds.clear();
    }
    handleCrawlRequest() {
        let self = this;
        return (function* () {
            const peers = [];
            for (const peerInfo of self.node.peerStore.peers.values()) {
                peers.push(peerInfo);
            }
            const filter = (peerInfo) => peerInfo.id.pubKey && !peerInfo.id.isEqual(self.node.peerInfo.id);
            const amountOfNodes = Math.min(constants_1.CRAWLING_RESPONSE_NODES, peers.length);
            const selectedNodes = hopr_utils_1.randomSubset(peers, amountOfNodes, filter);
            if (selectedNodes.length > 0) {
                yield new messages_1.CrawlResponse(undefined, {
                    status: messages_1.CrawlStatus.OK,
                    peerInfos: selectedNodes,
                });
            }
            else {
                yield new messages_1.CrawlResponse(undefined, {
                    status: messages_1.CrawlStatus.FAIL,
                });
            }
        })();
    }
    printStatsAndErrors(contactedPeerIds, errors, now, before) {
        if (errors.length > 0) {
            console.log(`Errors while crawling:${errors.reduce((acc, err) => {
                acc += `\n\t${chalk_1.default.red(err.message)}`;
                return acc;
            }, '')}`);
        }
        let contactedNodes = ``;
        contactedPeerIds.forEach((peerId) => {
            contactedNodes += `\n        ${peerId}`;
        });
        console.log(`Crawling results:\n    ${chalk_1.default.yellow(`contacted nodes:`)}: ${contactedNodes}\n    ${chalk_1.default.green(`new nodes`)}: ${now - before} node${now - before == 1 ? '' : 's'}\n    total: ${now} node${now == 1 ? '' : 's'}`);
    }
}
exports.Crawler = Crawler;
//# sourceMappingURL=crawler.js.map