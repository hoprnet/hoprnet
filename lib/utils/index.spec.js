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
const crypto_1 = require("crypto");
const secp256k1_1 = __importDefault(require("secp256k1"));
const utils = __importStar(require("."));
const u8a = __importStar(require("../core/u8a"));
const pair = {
    privKey: u8a.stringToU8a('0x9feaac2858974b0e16f6e3cfa7c21db6c7bbcd2094daa651ff3d5bb48a57b759'),
    pubKey: u8a.stringToU8a('0x03950056bd3c566eb3ac90b4e8cb0e93a648bf8000833161d679bd802505b224b5'),
    address: u8a.stringToU8a('0x81E1192eae6d7289A610956CaE1C4b76e083Eb39')
};
const generatePair = () => {
    // generate private key
    let privKey;
    do {
        privKey = crypto_1.randomBytes(32);
    } while (!secp256k1_1.default.privateKeyVerify(privKey));
    // get the public key in a compressed format
    const pubKey = secp256k1_1.default.publicKeyCreate(privKey);
    const address = secp256k1_1.default.publicKeyConvert(pubKey);
    return {
        privKey,
        pubKey,
        address
    };
};
const generateMsg = () => {
    return crypto_1.randomBytes(32);
};
describe('test utils', function () {
    it('should hash values', async function () {
        const testMsg = new Uint8Array([0, 0, 0, 0]);
        assert_1.default.deepEqual(await utils.hash(testMsg), 
        /* prettier-ignore */
        new Uint8Array([232, 231, 118, 38, 88, 111, 115, 185, 85, 54, 76, 123, 75, 191, 11, 183, 247, 104, 94, 189, 64, 232, 82, 177, 100, 99, 58, 74, 203, 211, 36, 76]));
    });
    it('should sign and verify signer', async function () {
        const { privKey, pubKey } = generatePair();
        const message = generateMsg();
        const signature = await utils.sign(message, privKey);
        const signer = await utils.signer(message, signature);
        assert_1.default(u8a.u8aEquals(pubKey, signer), `check that message is signed correctly`);
    });
    it('should sign and verify messages', async function () {
        const { privKey, pubKey } = generatePair();
        const message = generateMsg();
        const signature = await utils.sign(message, privKey);
        assert_1.default(await utils.verify(message, signature, pubKey), `check that signature is verifiable`);
        message[0] ^= 0xff;
        assert_1.default(!(await utils.verify(message, signature, pubKey)), `check that manipulated message is not verifiable`);
    });
    it('should get private key using public key', async function () {
        const pubKey = await utils.privKeyToPubKey(pair.privKey);
        assert_1.default(u8a.u8aEquals(pubKey, pair.pubKey));
    });
    it('should get address using public key', async function () {
        const address = await utils.pubKeyToAccountId(pair.pubKey);
        assert_1.default(u8a.u8aEquals(address, pair.address));
    });
});
