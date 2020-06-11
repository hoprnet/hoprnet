"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.TicketEpoch = exports.Ticket = exports.SignedTicket = exports.SignedChannel = exports.Signature = exports.State = exports.Public = exports.PreImage = exports.NativeBalance = exports.Moment = exports.Hash = exports.ChannelStatus = exports.ChannelEntry = exports.ChannelBalance = exports.ChannelId = exports.Channel = exports.Balance = exports.AccountId = void 0;
const accountId_1 = __importDefault(require("./accountId"));
exports.AccountId = accountId_1.default;
const balance_1 = __importDefault(require("./balance"));
exports.Balance = balance_1.default;
const channel_1 = __importStar(require("./channel"));
exports.Channel = channel_1.default;
Object.defineProperty(exports, "ChannelStatus", { enumerable: true, get: function () { return channel_1.ChannelStatus; } });
const channelBalance_1 = __importDefault(require("./channelBalance"));
exports.ChannelBalance = channelBalance_1.default;
const channelEntry_1 = __importDefault(require("./channelEntry"));
exports.ChannelEntry = channelEntry_1.default;
const channelId_1 = __importDefault(require("./channelId"));
exports.ChannelId = channelId_1.default;
const hash_1 = __importDefault(require("./hash"));
exports.Hash = hash_1.default;
const moment_1 = __importDefault(require("./moment"));
exports.Moment = moment_1.default;
const nativeBalance_1 = __importDefault(require("./nativeBalance"));
exports.NativeBalance = nativeBalance_1.default;
const preImage_1 = __importDefault(require("./preImage"));
exports.PreImage = preImage_1.default;
const public_1 = __importDefault(require("./public"));
exports.Public = public_1.default;
const signature_1 = __importDefault(require("./signature"));
exports.Signature = signature_1.default;
const signedChannel_1 = __importDefault(require("./signedChannel"));
exports.SignedChannel = signedChannel_1.default;
const signedTicket_1 = __importDefault(require("./signedTicket"));
exports.SignedTicket = signedTicket_1.default;
const state_1 = __importDefault(require("./state"));
exports.State = state_1.default;
const ticket_1 = __importDefault(require("./ticket"));
exports.Ticket = ticket_1.default;
const ticketEpoch_1 = __importDefault(require("./ticketEpoch"));
exports.TicketEpoch = ticketEpoch_1.default;
class Types {
    constructor() {
        this.AccountId = accountId_1.default;
        this.Balance = balance_1.default;
        this.Channel = channel_1.default;
        this.ChannelBalance = channelBalance_1.default;
        this.ChannelEntry = channelEntry_1.default;
        this.ChannelId = channelId_1.default;
        this.Hash = hash_1.default;
        this.Moment = moment_1.default;
        this.NativeBalance = nativeBalance_1.default;
        this.PreImage = preImage_1.default;
        this.Public = public_1.default;
        this.Signature = signature_1.default;
        this.SignedChannel = signedChannel_1.default;
        this.SignedTicket = signedTicket_1.default;
        this.State = state_1.default;
        this.Ticket = ticket_1.default;
        this.TicketEpoch = ticketEpoch_1.default;
    }
}
exports.default = Types;
//# sourceMappingURL=index.js.map