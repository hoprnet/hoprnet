"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.PacketAcknowledgementInteraction = void 0;
const it_pipe_1 = __importDefault(require("it-pipe"));
const chalk_1 = __importDefault(require("chalk"));
const acknowledgement_1 = require("../../messages/acknowledgement");
const events_1 = __importDefault(require("events"));
const constants_1 = require("../../constants");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
class PacketAcknowledgementInteraction extends events_1.default {
    constructor(node) {
        super();
        this.node = node;
        this.protocols = [constants_1.PROTOCOL_ACKNOWLEDGEMENT];
        this.node.handle(this.protocols, this.handler.bind(this));
    }
    handler(struct) {
        it_pipe_1.default(
        /* prettier-ignore */
        struct.stream, handleHelper.bind(this));
    }
    async interact(counterparty, acknowledgement) {
        let struct;
        try {
            struct = await this.node
                .dialProtocol(counterparty, this.protocols[0])
                .catch(async (err) => {
                return this.node.peerRouting
                    .findPeer(counterparty)
                    .then((peerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]));
            });
        }
        catch (err) {
            console.log(`Could not transfer acknowledgement to ${counterparty.toB58String()}. Error was: ${chalk_1.default.red(err.message)}.`);
            return;
        }
        await it_pipe_1.default(
        /* prettier-ignore */
        [acknowledgement], struct.stream);
    }
}
exports.PacketAcknowledgementInteraction = PacketAcknowledgementInteraction;
async function handleHelper(source) {
    let self = this;
    for await (const msg of source) {
        const arr = msg.slice();
        const acknowledgement = new acknowledgement_1.Acknowledgement(self.node.paymentChannels, {
            bytes: arr.buffer,
            offset: arr.byteOffset,
        });
        let record;
        const unAcknowledgedDbKey = hopr_utils_1.u8aToHex(self.node.dbKeys.UnAcknowledgedTickets(await acknowledgement.responseSigningParty, await acknowledgement.hashedKey));
        try {
            record = await self.node.db.get(unAcknowledgedDbKey);
            const acknowledgedDbKey = self.node.dbKeys.AcknowledgedTickets(await acknowledgement.responseSigningParty, acknowledgement.key);
            try {
                await self.node.db.batch().del(unAcknowledgedDbKey).put(acknowledgedDbKey, record).write();
            }
            catch (err) {
                console.log(`Error while writing to database. Error was ${chalk_1.default.red(err.message)}.`);
            }
        }
        catch (err) {
            if (err.notFound == true) {
                // console.log(
                //   `${chalk.blue(this.node.peerInfo.id.toB58String())} received unknown acknowledgement from party ${chalk.blue(
                //     (await pubKeyToPeerId(acknowledgement.responseSigningParty)).toB58String()
                //   )} for challenge ${chalk.yellow(u8aToHex(await acknowledgement.hashedKey))} - response was ${chalk.green(
                //     u8aToHex(await acknowledgement.hashedKey)
                //   )}. ${chalk.red('Dropping acknowledgement')}.`
                // )
            }
            else {
                self.node.log(`Database error: ${err.message}. ${chalk_1.default.red('Dropping acknowledgement')}.`);
            }
            continue;
        }
        finally {
            self.emit(unAcknowledgedDbKey);
        }
    }
}
//# sourceMappingURL=acknowledgement.js.map