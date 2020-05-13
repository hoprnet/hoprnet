"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.createHeader = void 0;
const secp256k1_1 = __importDefault(require("secp256k1"));
const crypto_1 = __importDefault(require("crypto"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const utils_1 = require("../../../utils");
const constants_1 = require("../../../constants");
const index_1 = require("./index");
const parameters_1 = require("./parameters");
async function createHeader(node, header, peerIds) {
    function checkPeerIds() {
        if (peerIds.length > constants_1.MAX_HOPS) {
            throw Error(`Expected at most ${constants_1.MAX_HOPS} but got ${peerIds.length}`);
        }
        peerIds.forEach((peerId, index) => {
            if (peerId.pubKey == null) {
                throw Error(`Invalid peerId at index ${index}.`);
            }
        });
    }
    function generateKeyShares() {
        let done = false;
        let secrets;
        let privKey;
        // Generate the Diffie-Hellman key shares and
        // the respective blinding factors for the
        // relays.
        // There exists a negligible, but NON-ZERO,
        // probability that the key share is chosen
        // such that it yields non-group elements.
        do {
            // initialize values
            let mul = new Uint8Array(parameters_1.PRIVATE_KEY_LENGTH);
            mul[parameters_1.PRIVATE_KEY_LENGTH - 1] = 1;
            const G = secp256k1_1.default.publicKeyCreate(mul);
            secrets = [];
            do {
                privKey = crypto_1.default.randomBytes(parameters_1.PRIVATE_KEY_LENGTH);
            } while (!secp256k1_1.default.privateKeyVerify(privKey));
            header.alpha.set(secp256k1_1.default.publicKeyCreate(privKey), 0);
            mul.set(privKey, 0);
            peerIds.forEach((peerId, index) => {
                // parallel
                // thread 1
                const alpha = secp256k1_1.default.publicKeyTweakMul(G, mul);
                // secp256k1.publicKeyVerify(alpha)
                // thread 2
                const secret = secp256k1_1.default.publicKeyTweakMul(peerId.pubKey.marshal(), mul);
                // secp256k1.publicKeyVerify(secret)
                // end parallel
                if (!secp256k1_1.default.publicKeyVerify(alpha) || !secp256k1_1.default.publicKeyVerify(secret)) {
                    return;
                }
                mul = secp256k1_1.default.privateKeyTweakMul(mul, index_1.deriveBlinding(alpha, secret));
                if (!secp256k1_1.default.privateKeyVerify(mul)) {
                    return;
                }
                secrets.push(secret);
                if (index == peerIds.length - 1) {
                    done = true;
                }
            });
        } while (!done);
        return secrets;
    }
    function generateFiller(secrets) {
        const filler = new Uint8Array(parameters_1.PER_HOP_SIZE * (secrets.length - 1));
        let length = 0;
        let start = parameters_1.LAST_HOP_SIZE + constants_1.MAX_HOPS * parameters_1.PER_HOP_SIZE;
        let end = parameters_1.LAST_HOP_SIZE + constants_1.MAX_HOPS * parameters_1.PER_HOP_SIZE;
        for (let index = 0; index < secrets.length - 1; index++) {
            let { key, iv } = index_1.derivePRGParameters(secrets[index]);
            start -= parameters_1.PER_HOP_SIZE;
            length += parameters_1.PER_HOP_SIZE;
            hopr_utils_1.u8aXOR(true, filler.subarray(0, length), utils_1.PRG.createPRG(key, iv).digest(start, end));
        }
        return filler;
    }
    async function createBetaAndGamma(secrets, filler, identifier) {
        const tmp = new Uint8Array(index_1.BETA_LENGTH - parameters_1.PER_HOP_SIZE);
        for (let i = secrets.length; i > 0; i--) {
            const { key, iv } = index_1.derivePRGParameters(secrets[i - 1]);
            let paddingLength = (constants_1.MAX_HOPS - secrets.length) * parameters_1.PER_HOP_SIZE;
            if (i == secrets.length) {
                header.beta.set(peerIds[i - 1].pubKey.marshal(), 0);
                header.beta.set(identifier, parameters_1.DESINATION_SIZE);
                // @TODO filling the array might not be necessary
                if (paddingLength > 0) {
                    header.beta.fill(0, parameters_1.LAST_HOP_SIZE, paddingLength);
                }
                hopr_utils_1.u8aXOR(true, header.beta.subarray(0, parameters_1.LAST_HOP_SIZE + paddingLength), utils_1.PRG.createPRG(key, iv).digest(0, parameters_1.LAST_HOP_SIZE + paddingLength));
                header.beta.set(filler, parameters_1.LAST_HOP_SIZE + paddingLength);
            }
            else {
                tmp.set(header.beta.subarray(0, index_1.BETA_LENGTH - parameters_1.PER_HOP_SIZE), 0);
                header.beta.set(peerIds[i].pubKey.marshal(), 0);
                header.beta.set(header.gamma, parameters_1.ADDRESS_SIZE);
                // Used for the challenge that is created for the next node
                header.beta.set(await node.paymentChannels.utils.hash(index_1.deriveTicketKeyBlinding(secrets[i])), parameters_1.ADDRESS_SIZE + parameters_1.MAC_SIZE);
                header.beta.set(tmp, parameters_1.PER_HOP_SIZE);
                if (i < secrets.length - 1) {
                    /**
                     * Tells the relay node which challenge it should for the issued ticket.
                     * The challenge should be done in a way such that:
                     *   - the relay node does not know how to solve it
                     *   - having one secret share is not sufficient to reconstruct
                     *     the secret
                     *   - the relay node can verify the key derivation path
                     */
                    header.beta.set(await node.paymentChannels.utils.hash(hopr_utils_1.u8aConcat(index_1.deriveTicketKey(secrets[i]), await node.paymentChannels.utils.hash(index_1.deriveTicketKeyBlinding(secrets[i + 1])))), parameters_1.ADDRESS_SIZE + parameters_1.MAC_SIZE + parameters_1.KEY_LENGTH);
                }
                else if (i == secrets.length - 1) {
                    header.beta.set(await node.paymentChannels.utils.hash(hopr_utils_1.u8aConcat(index_1.deriveTicketLastKey(secrets[i]), await node.paymentChannels.utils.hash(index_1.deriveTicketLastKeyBlinding(secrets[i])))), parameters_1.ADDRESS_SIZE + parameters_1.MAC_SIZE + parameters_1.KEY_LENGTH);
                }
                hopr_utils_1.u8aXOR(true, header.beta, utils_1.PRG.createPRG(key, iv).digest(0, index_1.BETA_LENGTH));
            }
            header.gamma.set(index_1.createMAC(secrets[i - 1], header.beta), 0);
        }
    }
    function toString(header, secrets) {
        return peerIds.reduce((str, peerId, index) => {
            str += `\nsecret[${index}]: ${hopr_utils_1.u8aToHex(secrets[index])}\npeerId[${index}]: ${peerId.toB58String()}\npeerId[${index}] pubkey: ${hopr_utils_1.u8aToHex(peerId.pubKey.marshal())}`;
            return str;
        }, header.toString());
    }
    checkPeerIds();
    const secrets = generateKeyShares();
    const identifier = crypto_1.default.randomBytes(parameters_1.IDENTIFIER_SIZE);
    const filler = generateFiller(secrets);
    await createBetaAndGamma(secrets, filler, identifier);
    // printValues(header, secrets)
    return {
        header: header,
        secrets: secrets,
        identifier: identifier,
    };
}
exports.createHeader = createHeader;
//# sourceMappingURL=createHeader.js.map