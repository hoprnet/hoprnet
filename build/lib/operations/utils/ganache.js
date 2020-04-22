"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const ganache_core_1 = __importDefault(require("ganache-core"));
const hopr_demo_seeds_1 = require("@hoprnet/hopr-demo-seeds");
const accounts = hopr_demo_seeds_1.NODE_SEEDS.concat(hopr_demo_seeds_1.BOOTSTRAP_SEEDS);
const balance = Number(1000000000000000000000000).toString(16);
let server;
const DEFAULT_OPS = {
    ws: true,
    port: 9545,
    accounts: accounts.map(account => ({
        secretKey: account,
        balance
    })),
    gasLimit: 0xfffffffffff,
    gasPrice: '1'
};
class CustomGanache {
    constructor(customOps = {}) {
        this.ops = {
            ...DEFAULT_OPS,
            ...customOps
        };
    }
    async start() {
        return new Promise((resolve, reject) => {
            console.log('Starting ganache instance');
            server = ganache_core_1.default.server(this.ops);
            server.listen(this.ops.port, err => {
                if (err)
                    return reject(err.message);
                const url = `${this.ops.ws ? 'ws' : 'http'}://127.0.0.1:${this.ops.port}`;
                console.log(`Network ready at ${url}`);
                return resolve(this);
            });
        });
    }
    async stop() {
        return new Promise((resolve, reject) => {
            console.log('Closing ganache instance');
            if (typeof server === 'undefined') {
                return resolve(this);
            }
            server.close(err => {
                if (err)
                    return reject(err.message);
                console.log('Network closed');
                server = undefined;
                return resolve(this);
            });
        });
    }
    async restart() {
        await this.stop();
        await this.start();
        return this;
    }
}
exports.default = CustomGanache;
