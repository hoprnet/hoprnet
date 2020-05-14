"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
var ganache_1 = require("./operations/utils/ganache");
Object.defineProperty(exports, "Ganache", { enumerable: true, get: function () { return ganache_1.default; } });
var fund_1 = require("./operations/fund");
Object.defineProperty(exports, "fund", { enumerable: true, get: function () { return fund_1.default; } });
var migrate_1 = require("./operations/migrate");
Object.defineProperty(exports, "migrate", { enumerable: true, get: function () { return migrate_1.default; } });
exports.addresses = __importStar(require("./addresses"));
