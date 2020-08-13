"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
const chalk_1 = __importDefault(require("chalk"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const types_1 = require("../types");
const utils_1 = require("../utils");
const config_1 = require("../config");
const heap_js_1 = __importDefault(require("heap-js"));
const SMALLEST_PUBLIC_KEY = new types_1.Public(hopr_utils_1.u8aConcat(new Uint8Array([0x02]), new Uint8Array(32).fill(0x00)));
const BIGGEST_PUBLIC_KEY = new types_1.Public(hopr_utils_1.u8aConcat(new Uint8Array([0x03]), new Uint8Array(32).fill(0xff)));
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
 * @returns true if blockNumber has passed max confirmations
 */
function isConfirmedBlock(blockNumber, onChainBlockNumber) {
    return blockNumber + config_1.MAX_CONFIRMATIONS <= onChainBlockNumber;
}
/**
 * Simple indexer to keep track of all open payment channels.
 */
class Indexer {
    constructor(connector) {
        this.connector = connector;
        this.log = utils_1.Log(['channels']);
        this.status = 'stopped';
        this.unconfirmedEvents = [];
    }
    /**
     * Returns the latest confirmed block number.
     *
     * @returns promive that resolves to a number
     */
    async getLatestConfirmedBlockNumber() {
        try {
            return hopr_utils_1.u8aToNumber(await this.connector.db.get(Buffer.from(this.connector.dbKeys.ConfirmedBlockNumber())));
        }
        catch (err) {
            if (err.notFound == null) {
                throw err;
            }
            return 0;
        }
    }
    /**
     * Check if channel entry exists.
     *
     * @returns promise that resolves to true or false
     */
    async has(partyA, partyB) {
        const { dbKeys, db } = this.connector;
        try {
            await db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB)));
        }
        catch (err) {
            if (err.notFound) {
                return false;
            }
        }
        return true;
    }
    /**
     * Get all stored channel entries, if party is provided,
     * it will return the open channels of the given party.
     *
     * @returns promise that resolves to a list of channel entries
     */
    async getAll(party) {
        const { dbKeys, db } = this.connector;
        const channels = [];
        return await new Promise((resolve, reject) => {
            db.createReadStream({
                gte: Buffer.from(dbKeys.ChannelEntry(SMALLEST_PUBLIC_KEY, SMALLEST_PUBLIC_KEY)),
                lte: Buffer.from(dbKeys.ChannelEntry(BIGGEST_PUBLIC_KEY, BIGGEST_PUBLIC_KEY)),
            })
                .on('error', (err) => reject(err))
                .on('data', ({ key, value }) => {
                const [partyA, partyB] = dbKeys.ChannelEntryParse(key);
                if (party != null && !(party.eq(partyA) || party.eq(partyB))) {
                    return;
                }
                const channelEntry = new types_1.ChannelEntry({
                    bytes: value,
                    offset: value.byteOffset,
                });
                channels.push({
                    partyA,
                    partyB,
                    channelEntry,
                });
            })
                .on('end', () => resolve(channels));
        });
    }
    /**
     * Get stored channel of the given parties.
     *
     * @returns promise that resolves to a channel entry or undefined if not found
     */
    async getSingle(partyA, partyB) {
        const { dbKeys, db } = this.connector;
        let _entry;
        try {
            _entry = await db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB)));
        }
        catch (err) {
            if (err.notFound) {
                return;
            }
        }
        if (_entry == null || _entry.length == 0) {
            return;
        }
        const channelEntry = new types_1.ChannelEntry({
            bytes: _entry,
            offset: _entry.byteOffset,
        });
        return {
            partyA,
            partyB,
            channelEntry,
        };
    }
    /**
     * Get stored channels entries.
     *
     * If query is left empty, it will return all channels.
     *
     * If only one party is provided, it will return all channels of the given party.
     *
     * If both parties are provided, it will return the channel of the given party.
     *
     * @param query
     * @returns promise that resolves to a list of channel entries
     */
    async get(query) {
        if (query == null) {
            // query not provided, get all channels
            return this.getAll();
        }
        else if (query.partyA != null && query.partyB != null) {
            // both parties provided, get channel
            const entry = await this.getSingle(query.partyA, query.partyB);
            if (typeof entry === 'undefined') {
                return [];
            }
            else {
                return [entry];
            }
        }
        else {
            // only one of the parties provided, get all open channels of party
            return this.getAll(query.partyA != null ? query.partyA : query.partyB);
        }
    }
    async store(partyA, partyB, channelEntry) {
        const { dbKeys, db } = this.connector;
        const { blockNumber, logIndex, transactionIndex } = channelEntry;
        this.log(`storing channel ${partyA.toHex()}-${partyB.toHex()}:${blockNumber.toString()}-${transactionIndex.toString()}-${logIndex.toString()}`);
        return await db.batch([
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
    async delete(partyA, partyB) {
        this.log(`deleting channel ${hopr_utils_1.u8aToHex(partyA)}-${hopr_utils_1.u8aToHex(partyB)}`);
        const { dbKeys, db } = this.connector;
        const key = Buffer.from(dbKeys.ChannelEntry(partyA, partyB));
        return db.del(key);
    }
    compareUnconfirmedEvents(a, b) {
        return a.blockNumber - b.blockNumber;
    }
    async onNewBlock(block) {
        while (this.unconfirmedEvents.length > 0 &&
            isConfirmedBlock(heap_js_1.default.heaptop(this.unconfirmedEvents, 1, this.compareUnconfirmedEvents)[0].blockNumber, block.number)) {
            const event = heap_js_1.default.heappop(this.unconfirmedEvents, this.compareUnconfirmedEvents);
            if (event.event === 'OpenedChannel') {
                this.onOpenedChannel(event);
            }
            else {
                this.onClosedChannel(event);
            }
        }
    }
    async onOpenedChannel(event) {
        let partyA, partyB;
        if (utils_1.isPartyA(await event.returnValues.opener.toAccountId(), await event.returnValues.counterparty.toAccountId())) {
            partyA = event.returnValues.opener;
            partyB = event.returnValues.counterparty;
        }
        else {
            partyA = event.returnValues.counterparty;
            partyB = event.returnValues.opener;
        }
        const newChannelEntry = new types_1.ChannelEntry(undefined, {
            blockNumber: new bn_js_1.default(event.blockNumber),
            transactionIndex: new bn_js_1.default(event.transactionIndex),
            logIndex: new bn_js_1.default(event.logIndex),
        });
        const channels = await this.get({
            partyA,
            partyB,
        });
        if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
            return;
        }
        this.store(partyA, partyB, newChannelEntry);
    }
    async onClosedChannel(event) {
        let partyA, partyB;
        if (utils_1.isPartyA(await event.returnValues.closer.toAccountId(), await event.returnValues.counterparty.toAccountId())) {
            partyA = event.returnValues.closer;
            partyB = event.returnValues.counterparty;
        }
        else {
            partyA = event.returnValues.counterparty;
            partyB = event.returnValues.closer;
        }
        const newChannelEntry = new types_1.ChannelEntry(undefined, {
            blockNumber: new bn_js_1.default(event.blockNumber),
            transactionIndex: new bn_js_1.default(event.transactionIndex),
            logIndex: new bn_js_1.default(event.logIndex),
        });
        const channels = await this.get({
            partyA,
            partyB,
        });
        if (channels.length === 0) {
            return;
        }
        else if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
            return;
        }
        await this.delete(partyA, partyB);
    }
    /**
     * Start indexing,
     * listen to all open / close events,
     * store entries after X confirmations.
     *
     * @returns true if start was succesful
     */
    async start() {
        this.log(`Starting indexer...`);
        if (typeof this.starting !== 'undefined') {
            return this.starting;
        }
        else if (typeof this.stopping !== 'undefined') {
            throw Error('cannot start while stopping');
        }
        else if (this.status === 'started') {
            return true;
        }
        this.starting = new Promise(async (resolve, reject) => {
            let rejected = false;
            try {
                const onChainBlockNumber = await this.connector.web3.eth.getBlockNumber();
                let fromBlock = await this.getLatestConfirmedBlockNumber();
                // go back 8 blocks in case of a re-org at time of stopping
                if (fromBlock - config_1.MAX_CONFIRMATIONS > 0) {
                    fromBlock = fromBlock - config_1.MAX_CONFIRMATIONS;
                }
                this.log(`starting to pull events from block ${fromBlock}..`);
                this.newBlockEvent = this.connector.web3.eth
                    .subscribe('newBlockHeaders')
                    .on('error', (err) => {
                    if (!rejected) {
                        rejected = true;
                        reject(err);
                    }
                })
                    .on('data', (block) => this.onNewBlock(block));
                this.openedChannelEvent = this.connector.web3.eth
                    .subscribe('logs', {
                    address: this.connector.hoprChannels.options.address,
                    fromBlock,
                    topics: utils_1.events.OpenedChannelTopics(undefined, undefined),
                })
                    .on('error', (err) => {
                    if (!rejected) {
                        rejected = true;
                        reject(err);
                    }
                })
                    .on('data', (_event) => this.processOpenedChannelEvent(_event, onChainBlockNumber));
                this.closedChannelEvent = this.connector.web3.eth
                    .subscribe('logs', {
                    address: this.connector.hoprChannels.options.address,
                    fromBlock,
                    topics: utils_1.events.ClosedChannelTopics(undefined, undefined),
                })
                    .on('error', (err) => {
                    if (!rejected) {
                        rejected = true;
                        reject(err);
                    }
                })
                    .on('data', (_event) => this.processClosedChannelEvent(_event, onChainBlockNumber));
                this.status = 'started';
                this.log(chalk_1.default.green('Indexer started!'));
                return resolve(true);
            }
            catch (err) {
                this.log(err.message);
                return this.stop();
            }
        }).finally(() => {
            this.starting = undefined;
        });
        return this.starting;
    }
    /**
     * Stop indexing.
     *
     * @returns true if stop was succesful
     */
    async stop() {
        this.log(`Stopping indexer...`);
        if (this.starting != null) {
            throw Error('cannot stop while starting');
        }
        else if (this.stopping != undefined) {
            return this.stopping;
        }
        else if (this.status === 'stopped') {
            return true;
        }
        this.stopping = new Promise((resolve) => {
            var _a, _b, _c;
            try {
                (_a = this.newBlockEvent) === null || _a === void 0 ? void 0 : _a.unsubscribe();
                (_b = this.openedChannelEvent) === null || _b === void 0 ? void 0 : _b.unsubscribe();
                (_c = this.closedChannelEvent) === null || _c === void 0 ? void 0 : _c.unsubscribe();
                this.status = 'stopped';
                this.log(chalk_1.default.green('Indexer stopped!'));
                return resolve(true);
            }
            catch (err) {
                this.log(err.message);
                return resolve(false);
            }
        }).finally(() => {
            this.stopping = undefined;
        });
        return this.stopping;
    }
    processOpenedChannelEvent(_event, onChainBlockNumber) {
        const event = utils_1.events.decodeOpenedChannelEvent(_event);
        if (isConfirmedBlock(event.blockNumber, onChainBlockNumber)) {
            this.onOpenedChannel(event);
        }
        else {
            // @TODO add membership with bloom filter to check before adding event to heap
            heap_js_1.default.heappush(this.unconfirmedEvents, event, this.compareUnconfirmedEvents);
        }
    }
    processClosedChannelEvent(_event, onChainBlockNumber) {
        const event = utils_1.events.decodeClosedChannelEvent(_event);
        if (isConfirmedBlock(event.blockNumber, onChainBlockNumber)) {
            this.onClosedChannel(event);
        }
        else {
            // @TODO add membership with bloom filter to check before adding event to heap
            heap_js_1.default.heappush(this.unconfirmedEvents, event, this.compareUnconfirmedEvents);
        }
    }
}
exports.default = Indexer;
//# sourceMappingURL=index.js.map