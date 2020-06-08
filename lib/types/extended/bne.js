"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
class BNE extends bn_js_1.default {
    toU8a(length) {
        return new Uint8Array(this.toBuffer('be', length));
    }
}
exports.default = BNE;
//# sourceMappingURL=bne.js.map