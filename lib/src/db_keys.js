"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const encoder = new TextEncoder();
const PREFIX = encoder.encode('tickets-');
const SEPERATOR = encoder.encode('-');
const acknowledgedSubPrefix = encoder.encode('acknowledged-');
const unAcknowledgedSubPrefix = encoder.encode('unacknowledged-');
const COMPRESSED_PUBLIC_KEY_LENGTH = 33;
const KEY_LENGTH = 32;
const utils_1 = require("./utils");
function AcknowledgedTickets(publicKeyCounterparty, id) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [acknowledgedSubPrefix.length, acknowledgedSubPrefix],
        [publicKeyCounterparty.length, publicKeyCounterparty],
        [SEPERATOR.length, SEPERATOR],
        [id.length, id]
    ]);
}
exports.AcknowledgedTickets = AcknowledgedTickets;
function UnAcknowledgedTickets(publicKeyCounterparty, id) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [unAcknowledgedSubPrefix.length, unAcknowledgedSubPrefix],
        [COMPRESSED_PUBLIC_KEY_LENGTH, publicKeyCounterparty],
        [SEPERATOR.length, SEPERATOR],
        [id.length, id]
    ]);
}
exports.UnAcknowledgedTickets = UnAcknowledgedTickets;
async function UnAcknowledgedTicketsParse(arg) {
    return [
        await utils_1.pubKeyToPeerId(arg.slice(PREFIX.length + unAcknowledgedSubPrefix.length, PREFIX.length + unAcknowledgedSubPrefix.length + COMPRESSED_PUBLIC_KEY_LENGTH)),
        arg.slice(PREFIX.length + unAcknowledgedSubPrefix.length + COMPRESSED_PUBLIC_KEY_LENGTH + SEPERATOR.length, PREFIX.length + unAcknowledgedSubPrefix.length + COMPRESSED_PUBLIC_KEY_LENGTH + SEPERATOR.length + KEY_LENGTH)
    ];
}
exports.UnAcknowledgedTicketsParse = UnAcknowledgedTicketsParse;
function allocationHelper(arr) {
    const totalLength = arr.reduce((acc, current) => {
        return acc + current[0];
    }, 0);
    let result = new Uint8Array(totalLength);
    let offset = 0;
    for (let [size, data] of arr) {
        result.set(data, offset);
        offset += size;
    }
    return result;
}
//# sourceMappingURL=db_keys.js.map