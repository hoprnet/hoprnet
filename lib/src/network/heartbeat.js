"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Heartbeat = void 0;
const peer_id_1 = __importDefault(require("peer-id"));
const events_1 = require("events");
const ONE_MINUTE = 1 * 60 * 1000;
const FORTY_ONE_SECONDS = 41 * 1000;
const REFRESH_TIME = ONE_MINUTE;
const CHECK_INTERVAL = FORTY_ONE_SECONDS;
const MAX_PARALLEL_CONNECTIONS = 10;
class Heartbeat extends events_1.EventEmitter {
    constructor(node) {
        super();
        this.node = node;
        this.heap = [];
        this.nodes = new Map();
        this.node.on('peer:connect', (peerInfo) => this.emit('beat', peerInfo.id));
        super.on('beat', this.connectionListener);
    }
    connectionListener(peer) {
        const peerIdString = peer.toB58String();
        let found = this.nodes.get(peerIdString);
        if (found == undefined) {
            this.heap.push(peerIdString);
        }
        this.nodes.set(peerIdString, Date.now());
    }
    comparator(a, b) {
        let lastSeenA = this.nodes.get(a);
        let lastSeenB = this.nodes.get(b);
        if (lastSeenA == lastSeenB) {
            return 0;
        }
        if (lastSeenA == undefined) {
            return 1;
        }
        if (lastSeenB == undefined) {
            return -1;
        }
        return lastSeenA < lastSeenB ? -1 : 1;
    }
    async checkNodes() {
        const promises = [];
        this.heap = this.heap.sort(this.comparator.bind(this));
        const THRESHOLD_TIME = Date.now() - REFRESH_TIME;
        // Remove non-existing nodes
        let index = this.heap.length - 1;
        while (this.nodes.get(this.heap[index--]) == undefined) {
            this.heap.pop();
        }
        let heapIndex = 0;
        const updateHeapIndex = () => {
            while (heapIndex < this.heap.length) {
                const lastSeen = this.nodes.get(this.heap[heapIndex]);
                if (lastSeen == undefined || lastSeen > THRESHOLD_TIME) {
                    heapIndex++;
                    continue;
                }
                else {
                    break;
                }
            }
        };
        const queryNode = async (startIndex) => {
            let currentPeerId;
            while (startIndex < this.heap.length) {
                currentPeerId = peer_id_1.default.createFromB58String(this.heap[startIndex]);
                try {
                    await this.node.interactions.network.heartbeat.interact(currentPeerId);
                    this.nodes.set(this.heap[startIndex], Date.now());
                }
                catch (err) {
                    this.nodes.delete(this.heap[startIndex]);
                    this.node.hangUp(currentPeerId);
                }
                startIndex = heapIndex;
            }
        };
        updateHeapIndex();
        while (promises.length < MAX_PARALLEL_CONNECTIONS && heapIndex < this.heap.length) {
            promises.push(queryNode(heapIndex++));
        }
        await Promise.all(promises);
    }
    start() {
        this.interval = setInterval(this.checkNodes.bind(this), CHECK_INTERVAL);
    }
    stop() {
        clearInterval(this.interval);
    }
}
exports.Heartbeat = Heartbeat;
//# sourceMappingURL=heartbeat.js.map