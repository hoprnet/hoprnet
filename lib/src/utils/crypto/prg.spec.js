"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const prg_1 = require("./prg");
const crypto_1 = require("crypto");
const assert_1 = __importDefault(require("assert"));
const u8a_1 = require("../u8a");
const general_1 = require("../general");
describe('Hopr Polkadot', async function () {
    it('should create a digest', function () {
        const [key, iv] = [crypto_1.randomBytes(prg_1.PRG.KEY_LENGTH), crypto_1.randomBytes(prg_1.PRG.IV_LENGTH)];
        const prg = prg_1.PRG.createPRG(key, iv);
        const digest = prg.digest(0, 500);
        const firstSlice = prg.digest(0, 32);
        assert_1.default.equal(firstSlice.length, 32, `check length`);
        assert_1.default(u8a_1.u8aEquals(firstSlice, digest.slice(0, 32)), `check that beginning is the same`);
        const start = general_1.randomInteger(0, 250);
        const end = general_1.randomInteger(start, start + 251);
        const secondSlice = prg.digest(start, end);
        assert_1.default.equal(secondSlice.length, end - start, `check size`);
        assert_1.default(u8a_1.u8aEquals(secondSlice, digest.slice(start, end)), `check that slice somewhere in the middle is the same`);
        assert_1.default(u8a_1.u8aEquals(prg_1.PRG.createPRG(key, iv).digest(start, end), prg.digest(start, end)), `check that slice somewhere in the middle is the same when computed by different methods`);
        assert_1.default.throws(() => prg.digest(234, 234), `should throw when start == end`);
        assert_1.default.throws(() => prg.digest(234, 233), `should throw when start > end`);
    });
});
