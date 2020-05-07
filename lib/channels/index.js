"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const dbKeys = __importStar(require("../dbKeys"));
const types_1 = require("../types");
const utils_1 = require("../utils");
const config_1 = require("../config");
const log = utils_1.Log(['channels']);
const unconfirmedEvents = new Map();
let isStarted = false;
let newBlockEvent;
let openedChannelEvent;
let closedChannelEvent;
/**
 * @returns a custom event id for logging purposes.
 */
function getEventId(event) {
    return `${event.event}-${event.transactionHash}-${event.transactionIndex}-${event.logIndex}`;
}
/**
 * Returns true if 'newChannelEntry' is more recent.
 *
 * @param oldChannelEntry
 * @param newChannelEntry
 * @returns true if 'newChannelEntry' is more recent than 'oldChannelEntry'
 */
function isMoreRecent(oldChannelEntry, newChannelEntry) {
    const okBlockNumber = oldChannelEntry.blockNumber.lte(newChannelEntry.blockNumber);
    const okTransactionIndex = okBlockNumber && oldChannelEntry.transactionIndex.lte(newChannelEntry.transactionIndex);
    const okLogIndex = okTransactionIndex && oldChannelEntry.logIndex.lt(newChannelEntry.logIndex);
    return okBlockNumber && okTransactionIndex && okLogIndex;
}
/**
 * Barebones indexer to keep track of all open payment channels.
 * Eventually we will move to a better solution.
 */
class Channels {
    static async getLatestConfirmedBlockNumber(connector) {
        try {
            const blockNumber = await connector.db.get(Buffer.from(connector.dbKeys.ConfirmedBlockNumber())).then((res) => {
                return hopr_utils_1.u8aToNumber(res);
            });
            return blockNumber;
        }
        catch (err) {
            if (err.notFound == null) {
                throw err;
            }
            return 0;
        }
    }
    // does it exist
    static async has(connector, partyA, partyB) {
        return connector.db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB))).then(() => true, (err) => {
            if (err.notFound) {
                return false;
            }
            else {
                throw err;
            }
        });
    }
    // @TODO: improve function types
    // get stored channels using a query
    static async get(connector, query) {
        const { dbKeys, db } = connector;
        const channels = [];
        const allSmall = new Uint8Array(types_1.AccountId.SIZE).fill(0x00);
        const allBig = new Uint8Array(types_1.AccountId.SIZE).fill(0xff);
        const hasQuery = typeof query !== 'undefined';
        const hasPartyA = hasQuery && typeof query.partyA !== 'undefined';
        const hasPartyB = hasQuery && typeof query.partyB !== 'undefined';
        if (hasQuery && !hasPartyA && !hasPartyB) {
            throw Error('query is empty');
        }
        let gte;
        let lte;
        if (hasQuery) {
            gte = Buffer.from(dbKeys.ChannelEntry(hasPartyA ? query.partyA : allSmall, hasPartyB ? query.partyB : allSmall));
            lte = Buffer.from(dbKeys.ChannelEntry(hasPartyA ? query.partyA : allBig, hasPartyB ? query.partyB : allBig));
        }
        else {
            gte = Buffer.from(dbKeys.ChannelEntry(allSmall, allSmall));
            lte = Buffer.from(dbKeys.ChannelEntry(allBig, allBig));
        }
        return new Promise((resolve, reject) => {
            db.createReadStream({
                gte,
                lte,
            })
                .on('error', (err) => reject(err))
                .on('data', ({ key, value }) => {
                const [partyA, partyB] = dbKeys.ChannelEntryParse(key);
                const channelEntry = new types_1.ChannelEntry({
                    bytes: value,
                    offset: value.byteOffset,
                });
                channels.push({
                    partyA: new types_1.AccountId(partyA),
                    partyB: new types_1.AccountId(partyB),
                    channelEntry,
                });
            })
                .on('end', () => resolve(channels));
        });
    }
    // get all stored channels
    static async getAll(connector) {
        return Channels.get(connector);
    }
    // store a channel
    static async store(connector, partyA, partyB, channelEntry) {
        const { dbKeys, db } = connector;
        const { blockNumber, logIndex, transactionIndex } = channelEntry;
        log(`storing channel ${partyA.toHex()}-${partyB.toHex()}:${blockNumber.toString()}-${transactionIndex.toString()}-${logIndex.toString()}`);
        return db.batch([
            {
                type: 'put',
                key: Buffer.from(dbKeys.ChannelEntry(partyA, partyB)),
                value: Buffer.from(channelEntry),
            },
            {
                type: 'put',
                key: Buffer.from(dbKeys.ConfirmedBlockNumber()),
                value: Buffer.from(blockNumber.toU8a()),
            },
        ]);
    }
    // delete a channel
    static async delete(connector, partyA, partyB) {
        log(`deleting channel ${partyA.toHex()}-${partyB.toHex()}`);
        const { dbKeys, db } = connector;
        const key = Buffer.from(dbKeys.ChannelEntry(partyA, partyB));
        return db.del(key);
    }
    static async onNewBlock(connector, block) {
        const confirmedEvents = Array.from(unconfirmedEvents.values()).filter((event) => {
            return event.blockNumber + config_1.MAX_CONFIRMATIONS <= block.number;
        });
        for (const event of confirmedEvents) {
            const id = getEventId(event);
            unconfirmedEvents.delete(id);
            if (event.event === 'OpenedChannel') {
                Channels.onOpenedChannel(connector, event);
            }
            else {
                Channels.onClosedChannel(connector, event);
            }
        }
    }
    static async onOpenedChannel(connector, event) {
        const opener = new types_1.AccountId(hopr_utils_1.stringToU8a(event.returnValues.opener));
        const counterParty = new types_1.AccountId(hopr_utils_1.stringToU8a(event.returnValues.counterParty));
        const [partyA, partyB] = utils_1.getParties(opener, counterParty);
        const newChannelEntry = new types_1.ChannelEntry(undefined, {
            blockNumber: new bn_js_1.default(event.blockNumber),
            transactionIndex: new bn_js_1.default(event.transactionIndex),
            logIndex: new bn_js_1.default(event.logIndex),
        });
        const channels = await Channels.get(connector, {
            partyA,
            partyB,
        });
        if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
            return;
        }
        Channels.store(connector, partyA, partyB, newChannelEntry);
    }
    static async onClosedChannel(connector, event) {
        const closer = new types_1.AccountId(hopr_utils_1.stringToU8a(event.returnValues.closer));
        const counterParty = new types_1.AccountId(hopr_utils_1.stringToU8a(event.returnValues.counterParty));
        const [partyA, partyB] = utils_1.getParties(closer, counterParty);
        const newChannelEntry = new types_1.ChannelEntry(undefined, {
            blockNumber: new bn_js_1.default(event.blockNumber),
            transactionIndex: new bn_js_1.default(event.transactionIndex),
            logIndex: new bn_js_1.default(event.logIndex),
        });
        const channels = await Channels.get(connector, {
            partyA,
            partyB,
        });
        if (channels.length === 0) {
            return;
        }
        else if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
            return;
        }
        Channels.delete(connector, partyA, partyB);
    }
    // listen to all open / close events, store entries after X confirmations
    static async start(connector) {
        try {
            if (isStarted) {
                log(`already started..`);
                return true;
            }
            let fromBlock = await Channels.getLatestConfirmedBlockNumber(connector);
            // go back 12 blocks in case of a re-org
            if (fromBlock - config_1.MAX_CONFIRMATIONS > 0) {
                fromBlock = fromBlock - config_1.MAX_CONFIRMATIONS;
            }
            log(`starting to pull events from block ${fromBlock}..`);
            newBlockEvent = connector.web3.eth.subscribe('newBlockHeaders').on('data', (block) => {
                Channels.onNewBlock(connector, block);
            });
            openedChannelEvent = connector.hoprChannels.events
                .OpenedChannel({
                fromBlock,
            })
                .on('data', (event) => {
                unconfirmedEvents.set(getEventId(event), event);
            });
            closedChannelEvent = connector.hoprChannels.events
                .ClosedChannel({
                fromBlock,
            })
                .on('data', (event) => {
                unconfirmedEvents.set(getEventId(event), event);
            });
            isStarted = true;
            return true;
        }
        catch (err) {
            log(err.message);
            return isStarted;
        }
    }
    // stop listening to events
    static async stop() {
        try {
            if (!isStarted)
                return true;
            if (typeof newBlockEvent !== 'undefined') {
                newBlockEvent.unsubscribe();
            }
            if (typeof openedChannelEvent !== 'undefined') {
                openedChannelEvent.removeAllListeners();
            }
            if (typeof closedChannelEvent !== 'undefined') {
                openedChannelEvent.removeAllListeners();
            }
            unconfirmedEvents.clear();
            isStarted = false;
            return true;
        }
        catch (err) {
            log(err.message);
            return isStarted;
        }
    }
}
exports.default = Channels;
