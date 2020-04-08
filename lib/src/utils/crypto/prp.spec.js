"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const prp_1 = require("./prp");
const u8a_1 = require("../u8a");
const assert_1 = __importDefault(require("assert"));
const crypto_1 = require("crypto");
describe(`test Pseudo-Random Generator`, function () {
    it(`should 'encrypt' and 'decrypt' a U8a`, function () {
        const prp = prp_1.PRP.createPRP(crypto_1.randomBytes(prp_1.PRP.KEY_LENGTH), crypto_1.randomBytes(prp_1.PRP.IV_LENGTH));
        const test = new Uint8Array(crypto_1.randomBytes(200)); // turn .slice() into copy
        const ciphertext = prp.permutate(test.slice());
        assert_1.default(ciphertext.some((value, index) => value != test[index]), 'ciphertext should be different from plaintext');
        const plaintext = prp.inverse(ciphertext);
        assert_1.default(u8a_1.u8aEquals(plaintext, test), `'encryption' and 'decryption' should yield the plaintext`);
    });
    it(`should 'decrypt' and 'encrypt' a U8a`, function () {
        const prp = prp_1.PRP.createPRP(crypto_1.randomBytes(prp_1.PRP.KEY_LENGTH), crypto_1.randomBytes(prp_1.PRP.IV_LENGTH));
        const test = new Uint8Array(crypto_1.randomBytes(200)); // turn .slice() into copy
        const ciphertext = prp.inverse(test.slice());
        assert_1.default(ciphertext.some((value, index) => value != test[index]), 'ciphertext should be different from plaintext');
        const plaintext = prp.permutate(ciphertext);
        assert_1.default(plaintext.every((value, index) => value == test[index]), `'decryption' and 'encryption' should yield the plaintext`);
    });
});
