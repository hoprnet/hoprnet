"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.createMAC = exports.deriveTicketLastKeyBlinding = exports.deriveTicketLastKey = exports.deriveTicketKeyBlinding = exports.deriveTicketKey = exports.deriveBlinding = exports.derivePRGParameters = exports.deriveCipherParameters = exports.deriveTagParameters = exports.BETA_LENGTH = exports.Header = void 0;
const secp256k1_1 = __importDefault(require("secp256k1"));
const futoin_hkdf_1 = __importDefault(require("futoin-hkdf"));
const crypto_1 = __importDefault(require("crypto"));
const createHeader_1 = require("./createHeader");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const constants_1 = require("../../../constants");
const parameters_1 = require("./parameters");
const MAC_KEY_LENGTH = 16;
const HASH_KEY_PRG = 'P';
const HASH_KEY_PRP = 'W';
const HASH_KEY_BLINDINGS = 'B';
const HASH_KEY_HMAC = 'H';
const HASH_KEY_TAGGING = 'T';
const HASH_KEY_TX = 'Tx';
const HASH_KEY_TX_BLINDED = 'Tx_';
const HASH_KEY_TX_LAST = 'Tx_Last';
const HASH_KEY_TX_LAST_BLINDED = 'Tx_Last_';
const TAG_SIZE = 16;
class Header extends Uint8Array {
    constructor(arr) {
        super(arr.bytes, arr.offset, Header.SIZE);
    }
    subarray(begin = 0, end = Header.SIZE) {
        return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin);
    }
    get alpha() {
        return this.subarray(0, parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH);
    }
    get beta() {
        return this.subarray(parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH, parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH + exports.BETA_LENGTH);
    }
    get gamma() {
        return this.subarray(parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH + exports.BETA_LENGTH, parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH + exports.BETA_LENGTH + parameters_1.MAC_SIZE);
    }
    get address() {
        return this.tmpData != null ? this.tmpData.subarray(0, parameters_1.ADDRESS_SIZE) : undefined;
    }
    get identifier() {
        return this.tmpData != null ? this.tmpData.subarray(parameters_1.ADDRESS_SIZE, parameters_1.ADDRESS_SIZE + parameters_1.IDENTIFIER_SIZE) : undefined;
    }
    get hashedKeyHalf() {
        return this.tmpData != null ? this.tmpData.subarray(parameters_1.ADDRESS_SIZE, parameters_1.ADDRESS_SIZE + parameters_1.KEY_LENGTH) : undefined;
    }
    get encryptionKey() {
        return this.tmpData != null ? this.tmpData.subarray(parameters_1.ADDRESS_SIZE + parameters_1.KEY_LENGTH, parameters_1.ADDRESS_SIZE + parameters_1.PROVING_VALUES_SIZE) : undefined;
    }
    get derivedSecret() {
        return this.tmpData != null
            ? this.derivedSecretLastNode != null
                ? this.derivedSecretLastNode
                : this.tmpData.subarray(parameters_1.ADDRESS_SIZE + parameters_1.PROVING_VALUES_SIZE, parameters_1.ADDRESS_SIZE + parameters_1.PROVING_VALUES_SIZE + parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH)
            : undefined;
    }
    deriveSecret(secretKey, lastNode = false) {
        if (!secp256k1_1.default.privateKeyVerify(secretKey)) {
            throw Error(`Invalid private key.`);
        }
        if (lastNode) {
            this.tmpData = this.beta.subarray(0, parameters_1.LAST_HOP_SIZE);
            this.derivedSecretLastNode = new Uint8Array(parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH);
        }
        else {
            this.tmpData = new Uint8Array(parameters_1.ADDRESS_SIZE + parameters_1.PROVING_VALUES_SIZE + parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH);
        }
        this.derivedSecret.set(secp256k1_1.default.publicKeyTweakMul(this.alpha, secretKey), 0);
    }
    verify() {
        return hopr_utils_1.u8aEquals(createMAC(this.derivedSecret, this.beta), this.gamma);
    }
    extractHeaderInformation(lastNode = false) {
        const { key, iv } = derivePRGParameters(this.derivedSecret);
        if (lastNode) {
            const { key, iv } = derivePRGParameters(this.derivedSecret);
            this.tmpData.set(hopr_utils_1.u8aXOR(false, this.beta.subarray(0, parameters_1.DESINATION_SIZE + parameters_1.IDENTIFIER_SIZE), hopr_utils_1.PRG.createPRG(key, iv).digest(0, parameters_1.DESINATION_SIZE + parameters_1.IDENTIFIER_SIZE)), 0);
        }
        else {
            const tmp = new Uint8Array(exports.BETA_LENGTH + parameters_1.PER_HOP_SIZE);
            tmp.set(this.beta, 0);
            tmp.fill(0, exports.BETA_LENGTH, exports.BETA_LENGTH + parameters_1.PER_HOP_SIZE);
            hopr_utils_1.u8aXOR(true, tmp, hopr_utils_1.PRG.createPRG(key, iv).digest(0, exports.BETA_LENGTH + parameters_1.PER_HOP_SIZE));
            // this.tmpData = this.tmpData || Buffer.alloc(ADDRESS_SIZE + PROVING_VALUES_SIZE + COMPRESSED_PUBLIC_KEY_LENGTH)
            this.tmpData.set(tmp.subarray(0, parameters_1.ADDRESS_SIZE), 0);
            this.tmpData.set(tmp.subarray(parameters_1.ADDRESS_SIZE + parameters_1.MAC_SIZE, parameters_1.PER_HOP_SIZE), parameters_1.ADDRESS_SIZE);
            this.gamma.set(tmp.subarray(parameters_1.ADDRESS_SIZE, parameters_1.ADDRESS_SIZE + parameters_1.MAC_SIZE), 0);
            this.beta.set(tmp.subarray(parameters_1.PER_HOP_SIZE, parameters_1.PER_HOP_SIZE + exports.BETA_LENGTH), 0);
        }
    }
    transformForNextNode() {
        if (this.tmpData == null) {
            throw Error(`Cannot read from 'this.data'. Please call 'deriveSecret()' first.`);
        }
        this.alpha.set(secp256k1_1.default.publicKeyTweakMul(this.alpha, deriveBlinding(this.alpha, this.derivedSecret)), 0);
    }
    toString() {
        return ('Header:\n' +
            '|-> Alpha:\n' +
            '|---> ' +
            hopr_utils_1.u8aToHex(this.alpha) +
            '\n' +
            '|-> Beta:\n' +
            '|---> ' +
            hopr_utils_1.u8aToHex(this.beta) +
            '\n' +
            '|-> Gamma:\n' +
            '|---> ' +
            hopr_utils_1.u8aToHex(this.gamma) +
            '\n');
    }
    static get SIZE() {
        return parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH + exports.BETA_LENGTH + parameters_1.MAC_SIZE;
    }
    static async create(node, peerIds, arr) {
        if (arr == null) {
            let tmpArray = new Uint8Array(Header.SIZE);
            arr = {
                bytes: tmpArray.buffer,
                offset: tmpArray.byteOffset
            };
        }
        const header = new Header(arr);
        header.tmpData = header.beta.subarray(parameters_1.ADDRESS_SIZE + parameters_1.MAC_SIZE, parameters_1.PER_HOP_SIZE);
        return createHeader_1.createHeader(node, header, peerIds);
    }
}
exports.Header = Header;
exports.BETA_LENGTH = parameters_1.PER_HOP_SIZE * (constants_1.MAX_HOPS - 1) + parameters_1.LAST_HOP_SIZE;
function deriveTagParameters(secret) {
    if (secret.length != parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
        throw Error('Secret must be a public key.');
    }
    return futoin_hkdf_1.default(Buffer.from(secret), TAG_SIZE, { salt: HASH_KEY_TAGGING });
}
exports.deriveTagParameters = deriveTagParameters;
function deriveCipherParameters(secret) {
    if (secret.length != parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
        throw Error('Secret must be a public key');
    }
    const keyAndIV = futoin_hkdf_1.default(Buffer.from(secret), hopr_utils_1.PRP.KEY_LENGTH + hopr_utils_1.PRP.IV_LENGTH, { salt: HASH_KEY_PRP });
    const key = keyAndIV.subarray(0, hopr_utils_1.PRP.KEY_LENGTH);
    const iv = keyAndIV.subarray(hopr_utils_1.PRP.KEY_LENGTH);
    return { key, iv };
}
exports.deriveCipherParameters = deriveCipherParameters;
function derivePRGParameters(secret) {
    if (secret.length != parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
        throw Error('Secret must be a public key');
    }
    const keyAndIV = futoin_hkdf_1.default(Buffer.from(secret), hopr_utils_1.PRG.KEY_LENGTH + hopr_utils_1.PRG.IV_LENGTH, { salt: HASH_KEY_PRG });
    const key = keyAndIV.subarray(0, hopr_utils_1.PRG.KEY_LENGTH);
    const iv = keyAndIV.subarray(hopr_utils_1.PRG.KEY_LENGTH, hopr_utils_1.PRG.KEY_LENGTH + hopr_utils_1.PRG.IV_LENGTH);
    return { key, iv };
}
exports.derivePRGParameters = derivePRGParameters;
function deriveBlinding(alpha, secret) {
    if (secret.length != parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
        throw Error('Secret must be a public key');
    }
    if (alpha.length != parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH || (alpha[0] != 0x02 && alpha[0] != 0x03)) {
        throw Error('Alpha must be a public key');
    }
    return futoin_hkdf_1.default(Buffer.from(hopr_utils_1.u8aConcat(alpha, secret)), parameters_1.PRIVATE_KEY_LENGTH, { salt: HASH_KEY_BLINDINGS });
}
exports.deriveBlinding = deriveBlinding;
function derivationHelper(secret, salt) {
    if (secret.length != parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
        throw Error('Secret must be a public key');
    }
    return futoin_hkdf_1.default(Buffer.from(secret), parameters_1.KEY_LENGTH, { salt });
}
function deriveTicketKey(secret) {
    return derivationHelper(secret, HASH_KEY_TX);
}
exports.deriveTicketKey = deriveTicketKey;
function deriveTicketKeyBlinding(secret) {
    return derivationHelper(secret, HASH_KEY_TX_BLINDED);
}
exports.deriveTicketKeyBlinding = deriveTicketKeyBlinding;
function deriveTicketLastKey(secret) {
    return derivationHelper(secret, HASH_KEY_TX_LAST);
}
exports.deriveTicketLastKey = deriveTicketLastKey;
function deriveTicketLastKeyBlinding(secret) {
    return derivationHelper(secret, HASH_KEY_TX_LAST_BLINDED);
}
exports.deriveTicketLastKeyBlinding = deriveTicketLastKeyBlinding;
function createMAC(secret, msg) {
    if (secret.length != parameters_1.COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
        throw Error('Secret must be a public key');
    }
    const key = futoin_hkdf_1.default(Buffer.from(secret), MAC_KEY_LENGTH, { salt: HASH_KEY_HMAC });
    return crypto_1.default
        .createHmac('sha256', key)
        .update(msg)
        .digest();
}
exports.createMAC = createMAC;
//# sourceMappingURL=index.js.map