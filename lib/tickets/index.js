"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const types_1 = require("../types");
/**
 * Store and get tickets stored by the node.
 */
class Tickets {
    static async store(coreConnector, channelId, signedTicket) {
        const { dbKeys, db } = coreConnector;
        const key = Buffer.from(dbKeys.Ticket(channelId, signedTicket.ticket.challenge));
        const value = Buffer.from(signedTicket);
        await db.put(key, value);
    }
    static async get(coreConnector, channelId) {
        const { dbKeys, db } = coreConnector;
        const tickets = new Map();
        return new Promise(async (resolve, reject) => {
            db.createReadStream({
                gte: Buffer.from(dbKeys.Ticket(channelId, new Uint8Array(types_1.SignedTicket.SIZE).fill(0x00))),
                lte: Buffer.from(dbKeys.Ticket(channelId, new Uint8Array(types_1.SignedTicket.SIZE).fill(0xff))),
            })
                .on('error', (err) => reject(err))
                .on('data', ({ value }) => {
                const signedTicket = new types_1.SignedTicket({
                    bytes: value.buffer,
                    offset: value.byteOffset,
                });
                tickets.set(signedTicket.ticket.challenge.toHex(), signedTicket);
            })
                .on('end', () => resolve(tickets));
        });
    }
}
exports.default = Tickets;
