"use strict";
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const types_1 = require("./types");
const constants = __importStar(require("./constants"));
const encoder = new TextEncoder();
const PREFIX = encoder.encode('payments-');
const SEPERATOR = encoder.encode('-');
const channelSubPrefix = encoder.encode('channel-');
const challengeSubPrefix = encoder.encode('challenge-');
function Channel(counterparty) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [channelSubPrefix.length, channelSubPrefix],
        [counterparty.length, counterparty]
    ]);
}
exports.Channel = Channel;
function ChannelKeyParse(arr) {
    return arr.slice(PREFIX.length + channelSubPrefix.length);
}
exports.ChannelKeyParse = ChannelKeyParse;
function Challenge(channelId, challenge) {
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [challengeSubPrefix.length, challengeSubPrefix],
        [channelId.length, channelId],
        [SEPERATOR.length, SEPERATOR],
        [challenge.length, challenge]
    ]);
}
exports.Challenge = Challenge;
function ChallengeKeyParse(arr) {
    return [
        new types_1.Hash(arr.slice(PREFIX.length + channelSubPrefix.length, PREFIX.length + channelSubPrefix.length + constants.HASH_LENGTH)),
        new types_1.Hash(arr.slice(PREFIX.length + channelSubPrefix.length + constants.HASH_LENGTH + SEPERATOR.length, PREFIX.length + channelSubPrefix.length + constants.HASH_LENGTH + SEPERATOR.length + constants.HASH_LENGTH))
    ];
}
exports.ChallengeKeyParse = ChallengeKeyParse;
function ChannelId(signatureHash) {
    const subPrefix = encoder.encode('channelId-');
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [subPrefix.length, subPrefix],
        [signatureHash.length, signatureHash]
    ]);
}
exports.ChannelId = ChannelId;
function Nonce(channelId, nonce) {
    const subPrefix = encoder.encode('nonce-');
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [subPrefix.length, subPrefix],
        [channelId.length, channelId],
        [SEPERATOR.length, SEPERATOR],
        [nonce.length, nonce]
    ]);
}
exports.Nonce = Nonce;
function OnChainSecret() {
    const subPrefix = encoder.encode('onChainSecret');
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [subPrefix.length, subPrefix]
    ]);
}
exports.OnChainSecret = OnChainSecret;
function Ticket(channelId, challenge) {
    const subPrefix = encoder.encode('ticket-');
    return allocationHelper([
        [PREFIX.length, PREFIX],
        [subPrefix.length, subPrefix],
        [channelId.length, channelId],
        [SEPERATOR.length, SEPERATOR],
        [challenge.length, challenge]
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
