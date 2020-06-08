"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.increaseTo = exports.increase = exports.latestBlock = exports.latest = exports.advanceBlockTo = exports.advanceBlock = void 0;
/*
  copied from OZ's text-helpers and modified to include a web3 param
  @TODO: find a way to re-use this through the original repo
*/
const util_1 = require("util");
const bn_js_1 = __importDefault(require("bn.js"));
function advanceBlock(web3) {
    // @ts-ignore
    return util_1.promisify(web3.currentProvider.send.bind(web3.currentProvider))({
        jsonrpc: '2.0',
        method: 'evm_mine',
        id: new Date().getTime(),
    });
}
exports.advanceBlock = advanceBlock;
// Advance the block to the passed height
async function advanceBlockTo(web3, target) {
    if (!bn_js_1.default.isBN(target)) {
        target = new bn_js_1.default(target);
    }
    const currentBlock = await latestBlock(web3);
    if (target.lt(currentBlock))
        throw Error(`Target block #(${target}) is lower than current block #(${currentBlock})`);
    while ((await latestBlock(web3)).lt(target)) {
        await advanceBlock(web3);
    }
}
exports.advanceBlockTo = advanceBlockTo;
// Returns the time of the last mined block in seconds
async function latest(web3) {
    const block = await web3.eth.getBlock('latest');
    return new bn_js_1.default(block.timestamp);
}
exports.latest = latest;
async function latestBlock(web3) {
    const block = await web3.eth.getBlock('latest');
    return new bn_js_1.default(block.number);
}
exports.latestBlock = latestBlock;
// Increases ganache time by the passed duration in seconds
async function increase(web3, duration) {
    if (!bn_js_1.default.isBN(duration)) {
        duration = new bn_js_1.default(duration);
    }
    if (duration.isNeg())
        throw Error(`Cannot increase time by a negative amount (${duration})`);
    // @ts-ignore
    await util_1.promisify(web3.currentProvider.send.bind(web3.currentProvider))({
        jsonrpc: '2.0',
        method: 'evm_increaseTime',
        params: [duration.toNumber()],
        id: new Date().getTime(),
    });
    await advanceBlock(web3);
}
exports.increase = increase;
/**
 * Beware that due to the need of calling two separate ganache methods and rpc calls overhead
 * it's hard to increase time precisely to a target point so design your test to tolerate
 * small fluctuations from time to time.
 *
 * @param target time in seconds
 */
async function increaseTo(web3, target) {
    if (!bn_js_1.default.isBN(target)) {
        target = new bn_js_1.default(target);
    }
    const now = await latest(web3);
    if (target.lt(now))
        throw Error(`Cannot increase current time (${now}) to a moment in the past (${target})`);
    const diff = target.sub(now);
    return increase(web3, diff);
}
exports.increaseTo = increaseTo;
//# sourceMappingURL=time.js.map