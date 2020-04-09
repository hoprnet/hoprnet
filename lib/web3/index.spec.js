"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const _1 = __importDefault(require("."));
const utils_1 = require("../utils");
const config_1 = require("../config");
const waitForEvent = (emitter, name) => {
    return new Promise(resolve => {
        emitter.on(name, () => {
            resolve();
        });
    });
};
describe('test custom web3', function () {
    this.timeout(1e3 * 5);
    const web3 = new _1.default(config_1.DEFAULT_URI);
    it('should connect and emit event', async function () {
        await utils_1.wait(1e3);
        assert_1.default(await web3.isConnected(), 'check isConnected method');
    });
    it('should disconnect and emit event', async function () {
        await Promise.all([waitForEvent(web3.events, 'disconnected'), web3.disconnect()]);
        assert_1.default(!(await web3.isConnected()), 'check disconnect method');
    });
    it('should reconnect and emit event', async function () {
        await Promise.all([waitForEvent(web3.events, 'connected'), web3.connect()]);
        const provider = web3.currentProvider;
        await Promise.all([
            waitForEvent(web3.events, 'connected'),
            waitForEvent(web3.events, 'reconnected'),
            provider.disconnect(0, 'client disconnected')
        ]);
        assert_1.default(await web3.isConnected(), 'check isConnected method');
    });
});
