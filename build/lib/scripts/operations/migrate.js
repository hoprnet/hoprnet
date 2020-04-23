"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const verify_1 = __importDefault(require("./verify"));
const utils_1 = require("./utils");
exports.default = async (network = 'development') => {
    await utils_1.bash(`npx truffle migrate --network ${network}`);
    if (!utils_1.isLocalNetwork(network)) {
        await verify_1.default(network, ...utils_1.getContractNames());
    }
};
