"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const toDecimalPoint_1 = require("./toDecimalPoint");
describe('test toDecimalPoint', function () {
    it('should result to 100', function () {
        assert_1.default.equal(toDecimalPoint_1.toDecimalPoint(1, 2), '100', 'check toDecimalPoint');
    });
    it('should result to 100', function () {
        assert_1.default.equal(toDecimalPoint_1.toDecimalPoint(0.01, 4), '100', 'check toDecimalPoint');
    });
    it('should result to 0.01', function () {
        assert_1.default.equal(toDecimalPoint_1.toDecimalPoint(1, -2), '0.01', 'check toDecimalPoint');
    });
    it('should result to 0.01', function () {
        assert_1.default.equal(toDecimalPoint_1.toDecimalPoint(100, -4), '0.01', 'check toDecimalPoint');
    });
});
