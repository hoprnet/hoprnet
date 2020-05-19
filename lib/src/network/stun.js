"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.StunServer = void 0;
const ministun = require("ministun");
class StunServer {
    constructor(options) {
        const config = {
            udp4: options.hosts.ip4 !== undefined,
            upd6: options.hosts.ip6 !== undefined,
            port: 3478,
            log: null,
            err: null,
            sw: true,
        };
        this.server = new ministun(config);
    }
    async start() {
        await this.server.start();
    }
    async stop() {
        await this.server.stop();
    }
}
exports.StunServer = StunServer;
//# sourceMappingURL=stun.js.map