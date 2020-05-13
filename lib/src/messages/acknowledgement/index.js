"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Acknowledgement = void 0;
const secp256k1_1 = __importDefault(require("secp256k1"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const header_1 = require("../packet/header");
const parameters_1 = require("../packet/header/parameters");
const challenge_1 = require("../packet/challenge");
/**
 * This class encapsulates the message that is sent back to the relayer
 * and allows that party to compute the key that is necessary to redeem
 * the previously received transaction.
 */
class Acknowledgement extends Uint8Array {
    constructor(paymentChannels, arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, Acknowledgement.SIZE(paymentChannels));
        }
        else if (arr == null && struct != null) {
            super(hopr_utils_1.u8aConcat(struct.key, struct.challenge, struct.signature != null
                ? struct.signature
                : new Uint8Array(paymentChannels.types.Signature.SIZE)));
        }
        else {
            throw Error('Invalid constructor parameters.');
        }
        this.paymentChannels = paymentChannels;
    }
    subarray(begin = 0, end = Acknowledgement.SIZE(this.paymentChannels)) {
        return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin);
    }
    get key() {
        return this.subarray(0, parameters_1.KEY_LENGTH);
    }
    get hashedKey() {
        if (this._hashedKey != null) {
            return Promise.resolve(this._hashedKey);
        }
        return this.paymentChannels.utils.hash(this.key).then((hashedKey) => {
            this._hashedKey = hashedKey;
            return hashedKey;
        });
    }
    get challenge() {
        return new challenge_1.Challenge(this.paymentChannels, {
            bytes: this.buffer,
            offset: this.byteOffset + parameters_1.KEY_LENGTH,
        });
    }
    get hash() {
        return this.paymentChannels.utils.hash(hopr_utils_1.u8aConcat(this.challenge, this.key));
    }
    get challengeSignatureHash() {
        return this.paymentChannels.utils.hash(this.challenge);
    }
    get challengeSigningParty() {
        return this.challenge.counterparty;
    }
    get responseSignature() {
        return this.paymentChannels.types.Signature.create({
            bytes: this.buffer,
            offset: this.byteOffset + parameters_1.KEY_LENGTH + challenge_1.Challenge.SIZE(this.paymentChannels),
        });
    }
    get responseSigningParty() {
        if (this._responseSigningParty != null) {
            return Promise.resolve(this._responseSigningParty);
        }
        return new Promise(async (resolve) => {
            this._responseSigningParty = secp256k1_1.default.ecdsaRecover(this.responseSignature.signature, this.responseSignature.recovery, this.responseSignature.msgPrefix != null && this.responseSignature.msgPrefix.length > 0
                ? await this.paymentChannels.utils.hash(hopr_utils_1.u8aConcat(this.responseSignature.msgPrefix, await this.hash))
                : await this.hash);
            resolve(this._responseSigningParty);
        });
    }
    async sign(peerId) {
        this.responseSignature.set(await this.paymentChannels.utils.sign(await this.hash, peerId.privKey.marshal(), peerId.pubKey.marshal()));
        return this;
    }
    async verify(peerId) {
        return this.paymentChannels.utils.verify(await this.hash, this.responseSignature, peerId.pubKey.marshal());
    }
    /**
     * Takes a challenge from a relayer and returns an acknowledgement that includes a
     * signature over the requested key half.
     *
     * @param challenge the signed challenge of the relayer
     * @param derivedSecret the secret that is used to create the second key half
     * @param signer contains private key
     */
    static async create(hoprCoreConnector, challenge, derivedSecret, signer) {
        const ack = new Acknowledgement(hoprCoreConnector, {
            bytes: new Uint8Array(Acknowledgement.SIZE(hoprCoreConnector)),
            offset: 0,
        });
        ack.key.set(header_1.deriveTicketKeyBlinding(derivedSecret));
        ack.challenge.set(challenge);
        return ack.sign(signer);
    }
    static SIZE(hoprCoreConnector) {
        return parameters_1.KEY_LENGTH + challenge_1.Challenge.SIZE(hoprCoreConnector) + hoprCoreConnector.types.Signature.SIZE;
    }
}
exports.Acknowledgement = Acknowledgement;
//# sourceMappingURL=index.js.map