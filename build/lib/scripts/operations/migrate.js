"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const utils_1 = require("./utils");
exports.default = async (network = 'development') => {
    await utils_1.bash(`npx truffle migrate --network ${network}`);
};
