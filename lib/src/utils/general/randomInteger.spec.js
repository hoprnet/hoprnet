"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const randomInteger_1 = require("./randomInteger");
describe('testing random-number generator', function () {
    let ATTEMPTS = 100;
    it(`should output values between '0' and '23'`, function () {
        let result = [];
        for (let i = 0; i < ATTEMPTS; i++) {
            result.push(randomInteger_1.randomInteger(23));
        }
        assert_1.default(result.every(value => 0 <= value && value < 23));
    });
    it(`should output values between '31' and '61'`, function () {
        let result = [];
        for (let i = 0; i < ATTEMPTS; i++) {
            result.push(randomInteger_1.randomInteger(31, 61));
        }
        assert_1.default(result.every(value => 31 <= value && value < 61));
    });
    it('should throw error for falsy interval input', function () {
        assert_1.default.throws(() => randomInteger_1.randomInteger(2, 1));
        assert_1.default.throws(() => randomInteger_1.randomInteger(Math.pow(2, 32)));
        assert_1.default.throws(() => randomInteger_1.randomInteger(-1));
        assert_1.default.throws(() => randomInteger_1.randomInteger(-1, -2));
    });
});
