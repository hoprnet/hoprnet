"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PacketInteractions = void 0;
const forward_1 = require("./forward");
const acknowledgement_1 = require("./acknowledgement");
class PacketInteractions {
    constructor(node) {
        this.acknowledgment = new acknowledgement_1.PacketAcknowledgementInteraction(node);
        this.forward = new forward_1.PacketForwardInteraction(node);
    }
}
exports.PacketInteractions = PacketInteractions;
//# sourceMappingURL=index.js.map