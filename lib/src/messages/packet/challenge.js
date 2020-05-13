"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Challenge = void 0;
const secp256k1_1 = __importDefault(require("secp256k1"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const KEY_LENGTH = 32;
/**
 * The purpose of this class is to give the relayer the opportunity to claim
 * the proposed funds in case the the next downstream node responds with an
 * inappropriate acknowledgement.
 */
class Challenge extends Uint8Array {
    constructor(paymentChannels, arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, Challenge.SIZE(paymentChannels));
        }
        else if (arr == null && struct != null) {
            super(struct.signature);
        }
        else {
            throw Error(`Invalid constructor parameters`);
        }
        this.paymentChannels = paymentChannels;
    }
    get challengeSignature() {
        return this.paymentChannels.types.Signature.create({
            bytes: this.buffer,
            offset: this.byteOffset,
        });
    }
    set challengeSignature(signature) {
        this.set(signature, 0);
    }
    get signatureHash() {
        return this.paymentChannels.utils.hash(this.challengeSignature);
    }
    static SIZE(paymentChannels) {
        return paymentChannels.types.Signature.SIZE;
    }
    get hash() {
        if (this._hashedKey == null) {
            throw Error(`Challenge was not set yet.`);
        }
        return this._hashedKey;
    }
    subarray(begin = 0, end = Challenge.SIZE(this.paymentChannels)) {
        return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin);
    }
    getCopy() {
        const arrCopy = new Uint8Array(Challenge.SIZE(this.paymentChannels));
        arrCopy.set(this);
        const copiedChallenge = new Challenge(this.paymentChannels, {
            bytes: arrCopy.buffer,
            offset: arrCopy.byteOffset,
        });
        copiedChallenge._hashedKey = this._hashedKey;
        copiedChallenge._fee = this._fee;
        return copiedChallenge;
    }
    /**
     * Uses the derived secret and the signature to recover the public
     * key of the signer.
     */
    get counterparty() {
        if (this._hashedKey == null) {
            return Promise.reject(Error(`Challenge was not set yet.`));
        }
        if (this._counterparty != null) {
            return Promise.resolve(this._counterparty);
        }
        return new Promise(async (resolve) => {
            resolve(secp256k1_1.default.ecdsaRecover(this.challengeSignature.signature, this.challengeSignature.recovery, this.challengeSignature.msgPrefix != null && this.challengeSignature.msgPrefix.length > 0
                ? await this.paymentChannels.utils.hash(hopr_utils_1.u8aConcat(this.challengeSignature.msgPrefix, await this.hash))
                : await this.hash));
        });
    }
    /**
     * Signs the challenge and includes the transferred amount of money as
     * well as the ethereum address of the signer into the signature.
     *
     * @param peerId that contains private key and public key of the node
     */
    async sign(peerId) {
        // const hashedChallenge = hash(Buffer.concat([this._hashedKey, this._fee.toBuffer('be', VALUE_LENGTH)], HASH_LENGTH + VALUE_LENGTH))
        const signature = await this.paymentChannels.utils.sign(await this.hash, peerId.privKey.marshal(), peerId.pubKey.marshal());
        this.challengeSignature = signature;
        return this;
    }
    /**
     * Creates a challenge object.
     *
     * @param hashedKey that is used to generate the key half
     * @param fee
     */
    static create(hoprCoreConnector, hashedKey, fee, arr) {
        if (hashedKey.length != KEY_LENGTH) {
            throw Error(`Invalid secret format. Expected a ${Uint8Array.name} of ${KEY_LENGTH} elements but got one with ${hashedKey.length}`);
        }
        if (arr == null) {
            const tmp = new Uint8Array(Challenge.SIZE(hoprCoreConnector));
            arr = {
                bytes: tmp.buffer,
                offset: tmp.byteOffset,
            };
        }
        const challenge = new Challenge(hoprCoreConnector, arr);
        challenge._hashedKey = hashedKey;
        challenge._fee = fee;
        return challenge;
    }
    /**
     * Verifies the challenge by checking whether the given public matches the
     * one restored from the signature.
     *
     * @param peerId PeerId instance that contains the public key of
     * the signer
     * @param secret the secret that was used to derive the key half
     */
    async verify(peerId) {
        if (!peerId.pubKey) {
            throw Error('Unable to verify challenge without a public key.');
        }
        return this.paymentChannels.utils.verify(this.hash, this.challengeSignature, peerId.pubKey.marshal());
    }
}
exports.Challenge = Challenge;
//# sourceMappingURL=challenge.js.map