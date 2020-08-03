"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
const peer_id_1 = __importDefault(require("peer-id"));
const utils_1 = require("../utils");
class Crawl {
    constructor(node) {
        this.node = node;
    }
    /**
     * Crawls the network to check for other nodes. Triggered by the CLI.
     */
    async execute() {
        try {
            await this.node.network.crawler.crawl((peer) => !utils_1.isBootstrapNode(this.node, peer_id_1.default.createFromB58String(peer)));
        }
        catch (err) {
            console.log(chalk_1.default.red(err.message));
        }
    }
    complete(line, cb) {
        cb(undefined, [[''], line]);
    }
}
exports.default = Crawl;
//# sourceMappingURL=crawl.js.map