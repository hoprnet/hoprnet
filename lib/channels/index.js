"use strict";
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
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
class Channels {
    static async getLatestConfirmedBlockNumber(coreConnector) {
        try {
            const blockNumber = await coreConnector.db
                .get(Buffer.from(coreConnector.dbKeys.ConfirmedBlockNumber()))
                .then((res) => {
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
    static async has(coreConnector, partyA, partyB) {
        return coreConnector.db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB))).then(() => true, (err) => {
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
    static async get(coreConnector, query) {
        const { dbKeys, db } = coreConnector;
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
                channels.push({
                    partyA: new types_1.AccountId(partyA),
                    partyB: new types_1.AccountId(partyB),
                    blockNumber: hopr_utils_1.u8aToNumber(value),
                });
            })
                .on('end', () => resolve(channels));
        });
    }
    // get all stored channels
    static async getAll(coreConnector) {
        return Channels.get(coreConnector);
    }
    // store a channel
    static async store(coreConnector, partyA, partyB, blockNumber) {
        log(`storing channel ${partyA.toHex()}-${partyB.toHex()}:${blockNumber}`);
        const { dbKeys, db } = coreConnector;
        return Promise.all([
            db.put(Buffer.from(dbKeys.ChannelEntry(partyA, partyB)), Buffer.from(hopr_utils_1.toU8a(blockNumber))),
            db.put(Buffer.from(dbKeys.ConfirmedBlockNumber()), Buffer.from(hopr_utils_1.toU8a(blockNumber))),
        ]);
    }
    // delete a channel
    static async delete(coreConnector, partyA, partyB) {
        log(`deleting channel ${partyA.toHex()}-${partyB.toHex()}`);
        const { dbKeys, db } = coreConnector;
        const key = Buffer.from(dbKeys.ChannelEntry(partyA, partyB));
        return db.del(key);
    }
    static async onNewBlock(coreConnector, block) {
        const confirmedEvents = Array.from(unconfirmedEvents.values()).filter((event) => {
            return event.blockNumber + config_1.MAX_CONFIRMATIONS <= block.number;
        });
        for (const event of confirmedEvents) {
            const id = utils_1.getEventId(event);
            unconfirmedEvents.delete(id);
            if (event.event === 'OpenedChannel') {
                Channels.onOpenedChannel(coreConnector, event);
            }
            else {
                Channels.onClosedChannel(coreConnector, event);
            }
        }
    }
    static async onOpenedChannel(coreConnector, event) {
        const opener = new types_1.AccountId(hopr_utils_1.stringToU8a(event.returnValues.opener));
        const counterParty = new types_1.AccountId(hopr_utils_1.stringToU8a(event.returnValues.counterParty));
        const [partyA, partyB] = utils_1.getParties(opener, counterParty);
        const channels = await Channels.get(coreConnector, {
            partyA,
            partyB,
        });
        if (channels.length > 0 && channels[0].blockNumber > event.blockNumber) {
            return;
        }
        Channels.store(coreConnector, partyA, partyB, event.blockNumber);
    }
    static async onClosedChannel(coreConnector, event) {
        const closer = new types_1.AccountId(hopr_utils_1.stringToU8a(event.returnValues.closer));
        const counterParty = new types_1.AccountId(hopr_utils_1.stringToU8a(event.returnValues.counterParty));
        const [partyA, partyB] = utils_1.getParties(closer, counterParty);
        const channels = await Channels.get(coreConnector, {
            partyA,
            partyB,
        });
        if (channels.length === 0) {
            return;
        }
        else if (channels.length > 0 && channels[0].blockNumber > event.blockNumber) {
            return;
        }
        Channels.delete(coreConnector, partyA, partyB);
    }
    // listen to all open / close events, store entries after X confirmations
    static async start(coreConnector) {
        try {
            if (isStarted) {
                log(`already started..`);
                return true;
            }
            let fromBlock = await Channels.getLatestConfirmedBlockNumber(coreConnector);
            // go back 12 blocks in case of a re-org
            if (fromBlock - config_1.MAX_CONFIRMATIONS > 0) {
                fromBlock = fromBlock - config_1.MAX_CONFIRMATIONS;
            }
            log(`starting to pull events from block ${fromBlock}..`);
            newBlockEvent = coreConnector.web3.eth.subscribe('newBlockHeaders').on('data', (block) => {
                Channels.onNewBlock(coreConnector, block);
            });
            openedChannelEvent = coreConnector.hoprChannels.events
                .OpenedChannel({
                fromBlock,
            })
                .on('data', (event) => {
                unconfirmedEvents.set(utils_1.getEventId(event), event);
            });
            closedChannelEvent = coreConnector.hoprChannels.events
                .ClosedChannel({
                fromBlock,
            })
                .on('data', (event) => {
                unconfirmedEvents.set(utils_1.getEventId(event), event);
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
