"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const closeChannel_1 = __importDefault(require("./closeChannel"));
const crawl_1 = __importDefault(require("./crawl"));
const listCommands_1 = __importDefault(require("./listCommands"));
const listConnectors_1 = __importDefault(require("./listConnectors"));
const listOpenChannels_1 = __importDefault(require("./listOpenChannels"));
const openChannel_1 = __importDefault(require("./openChannel"));
const ping_1 = __importDefault(require("./ping"));
const printAddress_1 = __importDefault(require("./printAddress"));
const printBalance_1 = __importDefault(require("./printBalance"));
const sendMessage_1 = __importDefault(require("./sendMessage"));
const stopNode_1 = __importDefault(require("./stopNode"));
const version_1 = __importDefault(require("./version"));
const tickets_1 = __importDefault(require("./tickets"));
class Commands {
    constructor(node) {
        this.node = node;
        this.closeChannel = new closeChannel_1.default(node);
        this.crawl = new crawl_1.default(node);
        this.listCommands = new listCommands_1.default();
        this.listConnectors = new listConnectors_1.default();
        this.listOpenChannels = new listOpenChannels_1.default(node);
        this.openChannel = new openChannel_1.default(node);
        this.ping = new ping_1.default(node);
        this.printAddress = new printAddress_1.default(node);
        this.printBalance = new printBalance_1.default(node);
        this.sendMessage = new sendMessage_1.default(node);
        this.stopNode = new stopNode_1.default(node);
        this.version = new version_1.default();
        this.tickets = new tickets_1.default(node);
    }
}
exports.default = Commands;
//# sourceMappingURL=index.js.map