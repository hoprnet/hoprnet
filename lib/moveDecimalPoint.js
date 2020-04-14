"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bignumber_js_1 = __importDefault(require("bignumber.js"));
exports.moveDecimalPoint = (amount, position) => {
    return new bignumber_js_1.default(amount).multipliedBy(new bignumber_js_1.default(10).pow(position)).toString();
};
