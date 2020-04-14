"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const moveDecimalPoint_1 = require("./moveDecimalPoint");
describe('test moveDecimalPoint', function () {
    it('should result to 100', function () {
        assert_1.default.equal(moveDecimalPoint_1.moveDecimalPoint(1, 2), '100', 'check moveDecimalPoint');
    });
    it('should result to 100', function () {
        assert_1.default.equal(moveDecimalPoint_1.moveDecimalPoint(0.01, 4), '100', 'check moveDecimalPoint');
    });
    it('should result to 0.01', function () {
        assert_1.default.equal(moveDecimalPoint_1.moveDecimalPoint(1, -2), '0.01', 'check moveDecimalPoint');
    });
    it('should result to 0.01', function () {
        assert_1.default.equal(moveDecimalPoint_1.moveDecimalPoint(100, -4), '0.01', 'check moveDecimalPoint');
    });
});
