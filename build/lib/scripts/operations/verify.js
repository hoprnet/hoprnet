"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const utils_1 = require("./utils");
exports.default = async (network = 'development', ...contractNamesArr) => {
    const contractNames = contractNamesArr.join(' ');
    await utils_1.bash(`npx truffle run verify ${contractNames} --network ${network}`);
};
