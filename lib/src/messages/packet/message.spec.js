"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const constants_1 = require("../../constants");
const message_1 = __importStar(require("./message"));
const utils_1 = require("../../utils");
const crypto_1 = require("crypto");
const parameters_1 = require("./header/parameters");
const secp256k1_1 = __importDefault(require("secp256k1"));
describe('test class that encapsulates (encrypted and padded) messages', function () {
    it('should create a Message object and encrypt / decrypt it', function () {
        const msg = message_1.default.createPlain('test');
        const testMessage = new TextEncoder().encode('test');
        assert_1.default(utils_1.u8aEquals(
        /* prettier-ignore */
        utils_1.u8aConcat(new Uint8Array([0, 0, 0, 4]), message_1.PADDING, testMessage, new Uint8Array(constants_1.PACKET_SIZE - message_1.PADDING.length - utils_1.LENGTH_PREFIX_LENGTH - testMessage.length)), msg));
        assert_1.default.throws(() => message_1.default.createPlain(new Uint8Array(constants_1.PACKET_SIZE - message_1.PADDING.length - utils_1.LENGTH_PREFIX_LENGTH + 1)));
        const secrets = [];
        for (let i = 0; i < 2; i++) {
            secrets.push(secp256k1_1.default.publicKeyCreate(crypto_1.randomBytes(parameters_1.PRIVATE_KEY_LENGTH)));
        }
        msg.onionEncrypt(secrets);
        secrets.forEach((secret) => {
            msg.decrypt(secret);
        });
        msg.encrypted = false;
        assert_1.default(utils_1.u8aEquals(msg.plaintext, testMessage));
    });
});
