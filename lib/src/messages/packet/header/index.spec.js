"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const _1 = require(".");
const hopr_core_polkadot_1 = require("@hoprnet/hopr-core-polkadot");
const peer_id_1 = __importDefault(require("peer-id"));
const crypto_1 = require("crypto");
const secp256k1_1 = __importDefault(require("secp256k1"));
const utils_1 = require("../../../utils");
describe('test creation & transformation of a header', async function () {
    async function createAndDecomposeHeader(node, peerIds) {
        const { header, identifier, secrets } = await _1.Header.create(node, peerIds);
        for (let i = 0; i < peerIds.length - 1; i++) {
            header.deriveSecret(peerIds[i].privKey.marshal());
            assert_1.default(utils_1.u8aEquals(header.derivedSecret, secrets[i]), `pre-computed secret and derived secret should be the same`);
            assert_1.default(header.verify(), `MAC must be valid`);
            header.extractHeaderInformation();
            assert_1.default(peerIds[i + 1].pubKey.marshal().every((value, index) => value == header.address[index]), `Decrypted address should be the same as the one of node ${i + 1}`);
            header.transformForNextNode();
        }
        return { header, identifier, secrets };
    }
    function getNode() {
        const node = {
            paymentChannels: {
                utils: hopr_core_polkadot_1.Utils
            }
        };
        return node;
    }
    it('should derive parameters', function () {
        const secret = crypto_1.randomBytes(32);
        const alpha = crypto_1.randomBytes(32);
        const secretGroupElement = secp256k1_1.default.publicKeyCreate(secret);
        const blinding = _1.deriveBlinding(secp256k1_1.default.publicKeyCreate(alpha), secretGroupElement);
        const encryptionKey = _1.deriveCipherParameters(secretGroupElement);
        const tagParameter = _1.deriveTagParameters(secretGroupElement);
        const mac = _1.createMAC(secretGroupElement, new TextEncoder().encode('test'));
        const transactionKey = _1.deriveTicketKey(secretGroupElement);
        const prgParameters = _1.derivePRGParameters(secretGroupElement);
        assert_1.default(notEqualHelper([blinding, encryptionKey.iv, encryptionKey.key, tagParameter, mac, transactionKey, prgParameters.key, prgParameters.iv]), 'Keys should all be with high probability different');
    });
    it('should create a header', async function () {
        const peerIds = await Promise.all([
            peer_id_1.default.create({ keyType: 'secp256k1' }),
            peer_id_1.default.create({ keyType: 'secp256k1' }),
            peer_id_1.default.create({ keyType: 'secp256k1' })
        ]);
        const { header, identifier, secrets } = await createAndDecomposeHeader(getNode(), peerIds);
        header.deriveSecret(peerIds[2].privKey.marshal(), true);
        assert_1.default(utils_1.u8aEquals(header.derivedSecret, secrets[2]), `pre-computed secret and derived secret should be the same`);
        assert_1.default(header.verify(), `MAC should be valid`);
        header.extractHeaderInformation(true);
        assert_1.default(utils_1.u8aEquals(peerIds[2].pubKey.marshal(), header.address), `Decrypted address should be the same as the final recipient`);
        assert_1.default(utils_1.u8aEquals(header.identifier, identifier), `Decrypted identifier should have the expected value`);
    });
    it('should create a header with a path less than MAX_HOPS nodes', async function () {
        const peerIds = await Promise.all([peer_id_1.default.create({ keyType: 'secp256k1' }), peer_id_1.default.create({ keyType: 'secp256k1' })]);
        const { header, identifier, secrets } = await createAndDecomposeHeader(getNode(), peerIds);
        header.deriveSecret(peerIds[1].privKey.marshal(), true);
        assert_1.default(utils_1.u8aEquals(header.derivedSecret, secrets[1]), `pre-computed secret and derived secret should be the same`);
        assert_1.default(header.verify(), `MAC must be valid`);
        header.extractHeaderInformation(true);
        assert_1.default(utils_1.u8aEquals(peerIds[1].pubKey.marshal(), header.address), `Decrypted address should be the same as the final recipient`);
        assert_1.default(utils_1.u8aEquals(header.identifier, identifier), `Decrypted identifier should have the expected value`);
    });
    it('should create a header with exactly two nodes', async function () {
        const peerIds = [await peer_id_1.default.create({ keyType: 'secp256k1' })];
        const { header, identifier, secrets } = await createAndDecomposeHeader(getNode(), peerIds);
        header.deriveSecret(peerIds[0].privKey.marshal(), true);
        assert_1.default(utils_1.u8aEquals(header.derivedSecret, secrets[0]), `pre-computed secret and derived secret should be the same`);
        assert_1.default(header.verify(), `MAC must be valid`);
        header.extractHeaderInformation(true);
        assert_1.default(utils_1.u8aEquals(peerIds[0].pubKey.marshal(), header.address), `Decrypted address should be the same as the final recipient`);
        assert_1.default(utils_1.u8aEquals(header.identifier, identifier), `Decrypted identifier should have the expected value`);
    });
});
function notEqualHelper(arr) {
    for (let i = 0; i < arr.length; i++) {
        for (let j = i + 1; j < arr.length; j++) {
            if (arr[i].length == arr[j].length && utils_1.u8aEquals(arr[i], arr[j])) {
                return false;
            }
        }
    }
    return true;
}
