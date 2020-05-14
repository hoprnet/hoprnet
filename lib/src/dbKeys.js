"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.KeyPair = exports.PacketTag = exports.UnAcknowledgedTicketsParse = exports.UnAcknowledgedTickets = exports.AcknowledgedTickets = void 0;
const utils_1 = require("./utils");
const encoder = new TextEncoder();
const TICKET_PREFIX = encoder.encode('tickets-');
const PACKET_PREFIX = encoder.encode('packets-');
const SEPERATOR = encoder.encode('-');
const acknowledgedSubPrefix = encoder.encode('acknowledged-');
const unAcknowledgedSubPrefix = encoder.encode('unacknowledged-');
const packetTagSubPrefix = encoder.encode('tag-');
const COMPRESSED_PUBLIC_KEY_LENGTH = 33;
const KEY_LENGTH = 32;
function AcknowledgedTickets(publicKeyCounterparty, id) {
    return allocationHelper([
        [TICKET_PREFIX.length, TICKET_PREFIX],
        [acknowledgedSubPrefix.length, acknowledgedSubPrefix],
        [publicKeyCounterparty.length, publicKeyCounterparty],
        [SEPERATOR.length, SEPERATOR],
        [id.length, id],
    ]);
}
exports.AcknowledgedTickets = AcknowledgedTickets;
function UnAcknowledgedTickets(publicKeyCounterparty, id) {
    return allocationHelper([
        [TICKET_PREFIX.length, TICKET_PREFIX],
        [unAcknowledgedSubPrefix.length, unAcknowledgedSubPrefix],
        [COMPRESSED_PUBLIC_KEY_LENGTH, publicKeyCounterparty],
        [SEPERATOR.length, SEPERATOR],
        [id.length, id],
    ]);
}
exports.UnAcknowledgedTickets = UnAcknowledgedTickets;
async function UnAcknowledgedTicketsParse(arg) {
    return [
        await utils_1.pubKeyToPeerId(arg.slice(TICKET_PREFIX.length + unAcknowledgedSubPrefix.length, TICKET_PREFIX.length + unAcknowledgedSubPrefix.length + COMPRESSED_PUBLIC_KEY_LENGTH)),
        arg.slice(TICKET_PREFIX.length +
            unAcknowledgedSubPrefix.length +
            COMPRESSED_PUBLIC_KEY_LENGTH +
            SEPERATOR.length, TICKET_PREFIX.length +
            unAcknowledgedSubPrefix.length +
            COMPRESSED_PUBLIC_KEY_LENGTH +
            SEPERATOR.length +
            KEY_LENGTH),
    ];
}
exports.UnAcknowledgedTicketsParse = UnAcknowledgedTicketsParse;
function PacketTag(tag) {
    return allocationHelper([
        [PACKET_PREFIX.length, PACKET_PREFIX],
        [packetTagSubPrefix.length, packetTagSubPrefix],
        [SEPERATOR.length, SEPERATOR],
        [tag.length, tag],
    ]);
}
exports.PacketTag = PacketTag;
exports.KeyPair = encoder.encode('keyPair');
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
//# sourceMappingURL=dbKeys.js.map