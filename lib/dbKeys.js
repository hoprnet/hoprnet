"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Ticket = exports.OnChainSecretIntermediary = exports.OnChainSecret = exports.Nonce = exports.ChannelId = exports.ChallengeKeyParse = exports.Challenge = exports.ChannelEntryParse = exports.ChannelEntry = exports.ConfirmedBlockNumber = exports.ChannelKeyParse = exports.Channel = void 0;
/*
  Helper functions which generate database keys
*/
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const types_1 = require("./types");
const encoder = new TextEncoder();
const PREFIX = encoder.encode('payments-');
const SEPERATOR = encoder.encode('-');
const channelSubPrefix = encoder.encode('channel-');
const channelEntrySubPrefix = encoder.encode('channelEntry-');
const challengeSubPrefix = encoder.encode('challenge-');
const channelIdSubPrefix = encoder.encode('channelId-');
const nonceSubPrefix = encoder.encode('nonce-');
const ticketSubPrefix = encoder.encode('ticket-');
const onChainSecretIntermediary = encoder.encode('onChainSecretIntermediary-');
const confirmedBlockNumber = encoder.encode('confirmedBlockNumber');
const ON_CHAIN_SECRET_ITERATION_WIDTH = 4; // bytes
/**
 * Returns the db-key under which the channel is saved.
 * @param counterparty counterparty of the channel
 */
function Channel(counterparty) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [channelSubPrefix.length, channelSubPrefix],
        [counterparty.length, counterparty],
    ]);
}
exports.Channel = Channel;
/**
 * Reconstructs the channelId from a db-key.
 * @param arr a channel db-key
 */
function ChannelKeyParse(arr) {
    return arr.slice(PREFIX.length + channelSubPrefix.length);
}
exports.ChannelKeyParse = ChannelKeyParse;
/**
 * Returns the db-key under which the latest confirmed block number is saved in the database.
 */
function ConfirmedBlockNumber() {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [confirmedBlockNumber.length, confirmedBlockNumber],
    ]);
}
exports.ConfirmedBlockNumber = ConfirmedBlockNumber;
/**
 * Returns the db-key under which channel entries are saved.
 * @param partyA the accountId of partyA
 * @param partyB the accountId of partyB
 */
function ChannelEntry(partyA, partyB) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [channelEntrySubPrefix.length, channelEntrySubPrefix],
        [types_1.Public.SIZE, partyA],
        [SEPERATOR.length, SEPERATOR],
        [types_1.Public.SIZE, partyB],
    ]);
}
exports.ChannelEntry = ChannelEntry;
/**
 * Reconstructs parties from a channel entry db-key.
 * @param arr a challenge db-key
 * @returns an array containing partyA's and partyB's accountIds
 */
function ChannelEntryParse(arr) {
    const partyAStart = PREFIX.length + channelEntrySubPrefix.length;
    const partyAEnd = partyAStart + types_1.Public.SIZE;
    const partyBStart = partyAEnd + SEPERATOR.length;
    const partyBEnd = partyBStart + types_1.Public.SIZE;
    return [new types_1.Public(arr.slice(partyAStart, partyAEnd)), new types_1.Public(arr.slice(partyBStart, partyBEnd))];
}
exports.ChannelEntryParse = ChannelEntryParse;
/**
 * Returns the db-key under which the challenge is saved.
 * @param channelId channelId of the channel
 * @param challenge challenge to save
 */
function Challenge(channelId, challenge) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [challengeSubPrefix.length, challengeSubPrefix],
        [types_1.Hash.SIZE, channelId],
        [SEPERATOR.length, SEPERATOR],
        [types_1.Hash.SIZE, challenge],
    ]);
}
exports.Challenge = Challenge;
/**
 * Reconstructs channelId and the specified challenge from a challenge db-key.
 * @param arr a challenge db-key
 */
function ChallengeKeyParse(arr) {
    const channelIdStart = PREFIX.length + challengeSubPrefix.length;
    const channelIdEnd = channelIdStart + types_1.Hash.SIZE;
    const challengeStart = channelIdEnd + SEPERATOR.length;
    const challengeEnd = challengeStart + types_1.Hash.SIZE;
    return [new types_1.Hash(arr.slice(channelIdStart, channelIdEnd)), new types_1.Hash(arr.slice(challengeStart, challengeEnd))];
}
exports.ChallengeKeyParse = ChallengeKeyParse;
/**
 * Returns the db-key under which signatures of acknowledgements are saved.
 * @param signatureHash hash of an ackowledgement signature
 */
function ChannelId(signatureHash) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [channelIdSubPrefix.length, channelIdSubPrefix],
        [types_1.Hash.SIZE, signatureHash],
    ]);
}
exports.ChannelId = ChannelId;
/**
 * Returns the db-key under which nonces are saved.
 * @param channelId channelId of the channel
 * @param nonce the nonce
 */
function Nonce(channelId, nonce) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [nonceSubPrefix.length, nonceSubPrefix],
        [types_1.Hash.SIZE, channelId],
        [SEPERATOR.length, SEPERATOR],
        [types_1.Hash.SIZE, nonce],
    ]);
}
exports.Nonce = Nonce;
function OnChainSecret() {
    return OnChainSecretIntermediary(0);
}
exports.OnChainSecret = OnChainSecret;
function OnChainSecretIntermediary(iteration) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [onChainSecretIntermediary.length, onChainSecretIntermediary],
        [SEPERATOR.length, SEPERATOR],
        [ON_CHAIN_SECRET_ITERATION_WIDTH, hopr_utils_1.toU8a(iteration, ON_CHAIN_SECRET_ITERATION_WIDTH)],
    ]);
}
exports.OnChainSecretIntermediary = OnChainSecretIntermediary;
/**
 * Returns the db-key under which the tickets are saved in the database.
 */
function Ticket(channelId, challenge) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [ticketSubPrefix.length, ticketSubPrefix],
        [types_1.Hash.SIZE, channelId],
        [SEPERATOR.length, SEPERATOR],
        [types_1.Hash.SIZE, challenge],
    ]);
}
exports.Ticket = Ticket;
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