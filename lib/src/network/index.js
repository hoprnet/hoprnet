"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Network = void 0;
const crawler_1 = require("./crawler");
const heartbeat_1 = require("./heartbeat");
const stun_1 = require("./stun");
class Network {
    constructor(node, options) {
        this.options = options;
        this.crawler = new crawler_1.Crawler(node);
        this.heartbeat = new heartbeat_1.Heartbeat(node);
        if (options.bootstrapNode) {
            this.stun = new stun_1.Stun(options);
        }
    }
    async start() {
        var _a, _b;
        if (this.options.bootstrapNode) {
            await ((_a = this.stun) === null || _a === void 0 ? void 0 : _a.startServer());
        }
        (_b = this.heartbeat) === null || _b === void 0 ? void 0 : _b.start();
    }
    async stop() {
        var _a, _b;
        if (this.options.bootstrapNode) {
            await ((_a = this.stun) === null || _a === void 0 ? void 0 : _a.stopServer());
        }
        (_b = this.heartbeat) === null || _b === void 0 ? void 0 : _b.stop();
    }
}
exports.Network = Network;
//# sourceMappingURL=index.js.map