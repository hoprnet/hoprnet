"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Interactions = void 0;
const payments_1 = require("./payments");
const network_1 = require("./network");
const packet_1 = require("./packet");
class Interactions {
    constructor(node) {
        this.payments = new payments_1.PaymentInteractions(node);
        this.network = new network_1.NetworkInteractions(node);
        this.packet = new packet_1.PacketInteractions(node);
    }
}
exports.Interactions = Interactions;
//# sourceMappingURL=index.js.map