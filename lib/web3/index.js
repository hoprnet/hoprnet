"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const events_1 = __importDefault(require("events"));
const web3_1 = __importDefault(require("web3"));
const utils_1 = require("../utils");
var Events;
(function (Events) {
    Events["connected"] = "connected";
    Events["disconnected"] = "disconnected";
    Events["reconnected"] = "reconnected";
})(Events = exports.Events || (exports.Events = {}));
exports.isConnectionError = (err) => {
    return err.message.includes('connection not open') || err.message.includes('CONNECTION ERROR');
};
class CustomWeb3 extends web3_1.default {
    constructor(uri, ops = {
        reconnection: true,
        reconnectionDelay: 1000
    }) {
        super();
        this.uri = uri;
        this.ops = ops;
        this.reconnecting = false;
        this.manualDisconnect = false;
        this.events = new events_1.default();
        // @TODO: find a better way to do this
        this.connect();
    }
    disconnected() {
        console.log('web3 disconnected!');
        this.events.emit('disconnected');
        if (this.manualDisconnect)
            return;
        if (this.reconnecting)
            return;
        return this.reconnect();
    }
    // @TODO: add max retries & increasing timer
    async reconnect() {
        try {
            // @TODO: should return promise
            if (this.reconnecting)
                return false;
            this.reconnecting = true;
            console.log('web3 reconnecting..');
            await utils_1.wait(this.ops.reconnectionDelay);
            return this.connect();
        }
        catch (err) {
            throw err;
        }
        finally {
            this.reconnecting = false;
        }
    }
    async isConnected() {
        return new Promise(async (resolve, reject) => {
            const currentProvider = this.currentProvider;
            if (!currentProvider)
                return resolve(false);
            try {
                const isListening = await this.eth.net.isListening();
                return resolve(isListening);
            }
            catch (err) {
                if (err.message.includes('connection not open')) {
                    return resolve(false);
                }
                return reject(err);
            }
        });
    }
    // @TODO: add timeout to 'isConnected'
    async connect() {
        return new Promise(async (resolve, reject) => {
            try {
                if (await this.isConnected()) {
                    return true;
                }
                const currentProvider = this.currentProvider;
                const provider = new web3_1.default.providers.WebsocketProvider(this.uri);
                provider.on('error', this.disconnected.bind(this));
                provider.on('end', this.disconnected.bind(this));
                this.manualDisconnect = false;
                this.setProvider(provider);
                while (!(await this.isConnected())) {
                    utils_1.wait(100);
                }
                if (typeof currentProvider !== 'undefined') {
                    this.events.emit('reconnected');
                }
                this.events.emit('connected');
                return resolve(true);
            }
            catch (err) {
                return reject(err);
            }
        });
    }
    async disconnect() {
        const provider = this.currentProvider;
        if (!provider)
            return;
        this.manualDisconnect = true;
        provider.disconnect(0, 'client disconnected manually');
        this.events.emit('disconnected');
    }
}
exports.default = CustomWeb3;
