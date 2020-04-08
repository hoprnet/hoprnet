"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const it_pipe_1 = __importDefault(require("it-pipe"));
const chalk_1 = __importDefault(require("chalk"));
const constants_1 = require("../../constants");
const messages_1 = require("../../messages");
class Crawler {
    constructor(node) {
        this.node = node;
        this.protocols = [constants_1.PROTOCOL_CRAWLING];
        this.node.handle(this.protocols, this.handler.bind(this));
    }
    handler(struct) {
        it_pipe_1.default(
        /* prettier-ignore */
        this.node.network.crawler.handleCrawlRequest(), struct.stream);
    }
    async interact(counterparty) {
        let struct;
        try {
            struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (_) => {
                return this.node.peerRouting.findPeer(counterparty.id).then((peerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]));
            });
        }
        catch (err) {
            this.node.log(`Could not ask node ${counterparty.id.toB58String()} for other nodes. Error was: ${chalk_1.default.red(err.message)}.`);
            return [];
        }
        return await it_pipe_1.default(
        /** prettier-ignore */
        struct.stream, collect);
    }
}
exports.Crawler = Crawler;
async function collect(source) {
    const peerInfos = [];
    for await (const encodedResponse of source) {
        let decodedResponse;
        try {
            decodedResponse = new messages_1.CrawlResponse(encodedResponse.slice());
        }
        catch {
            continue;
        }
        if (decodedResponse.status !== messages_1.CrawlStatus.OK) {
            continue;
        }
        peerInfos.push(...(await decodedResponse.peerInfos));
    }
    return peerInfos;
}
