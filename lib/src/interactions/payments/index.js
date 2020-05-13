"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PaymentInteractions = void 0;
const open_1 = require("./open");
const onChainKey_1 = require("./onChainKey");
class PaymentInteractions {
    constructor(node) {
        this.open = new open_1.Opening(node);
        this.onChainKey = new onChainKey_1.OnChainKey(node);
    }
}
exports.PaymentInteractions = PaymentInteractions;
//# sourceMappingURL=index.js.map