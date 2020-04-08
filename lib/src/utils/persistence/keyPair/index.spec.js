"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const serialize_1 = require("./serialize");
const deserialize_1 = require("./deserialize");
const rlp_1 = require("rlp");
const peer_id_1 = __importDefault(require("peer-id"));
const crypto_1 = require("crypto");
const assert_1 = __importDefault(require("assert"));
const general_1 = require("../../general");
const u8a_1 = require("../../u8a");
describe('test serialisation and deserialisation of encrypted keypair', function () {
    it('should serialize and deserialize a keypair', async function () {
        this.timeout(5000);
        const password = crypto_1.randomBytes(general_1.randomInteger(1, 33));
        const peerId = await peer_id_1.default.create({ keyType: 'secp256k1' });
        assert_1.default(!u8a_1.u8aEquals(await serialize_1.serializeKeyPair(peerId, password), await serialize_1.serializeKeyPair(peerId, password)), 'Serialization of same peerId should lead to different ciphertexts');
        const serializedKeyPair = await serialize_1.serializeKeyPair(peerId, password);
        assert_1.default(u8a_1.u8aEquals((await deserialize_1.deserializeKeyPair(serializedKeyPair, password)).marshal(), peerId.marshal()), 'PeerId must be recoverable from serialized peerId');
        const [salt, mac, encodedCiphertext] = rlp_1.decode(serializedKeyPair);
        try {
            const manipulatedSalt = Buffer.from(salt);
            manipulatedSalt.set(crypto_1.randomBytes(1), general_1.randomInteger(0, manipulatedSalt.length));
            await deserialize_1.deserializeKeyPair(rlp_1.encode([manipulatedSalt, mac, encodedCiphertext]), password);
            assert_1.default.fail('Shoud fail with manipulated salt');
        }
        catch { }
        try {
            const manipulatedMac = Buffer.from(salt);
            manipulatedMac.set(crypto_1.randomBytes(1), general_1.randomInteger(0, manipulatedMac.length));
            await deserialize_1.deserializeKeyPair(rlp_1.encode([salt, manipulatedMac, encodedCiphertext]), password);
            assert_1.default.fail('Shoud fail with manipulated MAC');
        }
        catch { }
        try {
            const manipulatedCiphertext = Buffer.from(salt);
            manipulatedCiphertext.set(crypto_1.randomBytes(1), general_1.randomInteger(0, manipulatedCiphertext.length));
            await deserialize_1.deserializeKeyPair(rlp_1.encode([salt, mac, manipulatedCiphertext]), password);
            assert_1.default.fail('Shoud fail with manipulated ciphertext');
        }
        catch { }
    });
});
