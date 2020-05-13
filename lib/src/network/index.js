"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Network = void 0;
const crawler_1 = require("./crawler");
const heartbeat_1 = require("./heartbeat");
class Network {
    constructor(node) {
        this.crawler = new crawler_1.Crawler(node);
        this.heartbeat = new heartbeat_1.Heartbeat(node);
    }
}
exports.Network = Network;
//# sourceMappingURL=index.js.map