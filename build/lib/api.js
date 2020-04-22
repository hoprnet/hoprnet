"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
// export { default as fund } from './fund'
var migrate_1 = require("./operations/migrate");
exports.migrate = migrate_1.default;
var ganache_1 = require("./operations/utils/ganache");
exports.Ganache = ganache_1.default;
