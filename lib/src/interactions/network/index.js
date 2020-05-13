"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.NetworkInteractions = void 0;
const crawler_1 = require("./crawler");
const forwardPacket_1 = require("./forwardPacket");
const heartbeat_1 = require("./heartbeat");
class NetworkInteractions {
    constructor(node) {
        this.crawler = new crawler_1.Crawler(node);
        this.forwardPacket = new forwardPacket_1.ForwardPacketInteraction(node);
        this.heartbeat = new heartbeat_1.Heartbeat(node);
    }
}
exports.NetworkInteractions = NetworkInteractions;
//# sourceMappingURL=index.js.map