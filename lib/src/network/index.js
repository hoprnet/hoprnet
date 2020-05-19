"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Network = void 0;
const crawler_1 = require("./crawler");
const heartbeat_1 = require("./heartbeat");
const stun_1 = require("./stun");
class Network {
    constructor(node, options) {
        this.crawler = new crawler_1.Crawler(node);
        this.heartbeat = new heartbeat_1.Heartbeat(node);
        if (options.bootstrapNode) {
            this.stun = new stun_1.StunServer(options);
        }
    }
    async start() {
        // await this.stun?.start()
        var _a;
        (_a = this.heartbeat) === null || _a === void 0 ? void 0 : _a.start();
    }
    async stop() {
        // await this.stun?.stop()
        var _a;
        (_a = this.heartbeat) === null || _a === void 0 ? void 0 : _a.stop();
    }
}
exports.Network = Network;
//# sourceMappingURL=index.js.map