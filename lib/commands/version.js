"use strict";
var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, privateMap) {
    if (!privateMap.has(receiver)) {
        throw new TypeError("attempted to get private field on non-instance");
    }
    return privateMap.get(receiver);
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var _display;
Object.defineProperty(exports, "__esModule", { value: true });
const package_json_1 = __importDefault(require("../package.json"));
class Version {
    constructor() {
        _display.set(this, [`hopr-chat: ${package_json_1.default.version}`, `hopr-core: ${package_json_1.default.dependencies['@hoprnet/hopr-core']}`].join('\n'));
    }
    async execute() {
        console.log(__classPrivateFieldGet(this, _display));
    }
    complete() { }
}
exports.default = Version;
_display = new WeakMap();
//# sourceMappingURL=version.js.map