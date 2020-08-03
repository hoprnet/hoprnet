"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const path_1 = require("path");
const read_pkg_up_1 = __importDefault(require("read-pkg-up"));
const pkg = read_pkg_up_1.default.sync({
    cwd: path_1.join(__dirname, '..'),
});
const corePkg = read_pkg_up_1.default.sync({
    cwd: path_1.join(__dirname, '..', 'node_modules', '@hoprnet', 'hopr-core'),
});
console.log(pkg.packageJson);
console.log(corePkg.packageJson);
exports.default = {
    // chat
    '@hoprnet/hopr-chat': pkg.packageJson.version,
    '@hoprnet/hopr-core': pkg.packageJson.dependencies['@hoprnet/hopr-core'],
    '@hoprnet/hopr-utils': pkg.packageJson.dependencies['@hoprnet/hopr-utils'],
    '@hoprnet/hopr-core-connector-interface': pkg.packageJson.dependencies['@hoprnet/hopr-core-connector-interface'],
    // core
    '@hoprnet/hopr-core-ethereum': corePkg.packageJson.dependencies['@hoprnet/hopr-core-ethereum'],
};
//# sourceMappingURL=dependancies.js.map