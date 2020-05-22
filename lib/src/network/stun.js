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
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Stun = void 0;
const dgram_1 = __importDefault(require("dgram"));
const stun = __importStar(require("webrtc-stun"));
const constants_1 = require("../constants");
class Stun {
    constructor(options) {
        this.options = options;
    }
    static getExternalIP(address, usePort) {
        return new Promise(async (resolve, reject) => {
            const socket = dgram_1.default.createSocket({ type: 'udp4' });
            const tid = stun.generateTransactionId();
            if (usePort !== undefined) {
                await bindSocketToPort(socket, usePort);
            }
            socket.on('message', async (msg) => {
                const res = stun.createBlank();
                if (res.loadBuffer(msg)) {
                    if (res.isBindingResponseSuccess({ transactionId: tid })) {
                        const attr = res.getXorMappedAddressAttribute();
                        if (attr) {
                            await releaseSocketFromPort(socket);
                            resolve(attr);
                        }
                    }
                }
            });
            const req = stun.createBindingRequest(tid).setFingerprintAttribute();
            socket.send(req.toBuffer(), address.port, address.hostname);
        });
    }
    getSocket() {
        if (this.options.hosts.ip4 !== undefined && this.options.hosts.ip6 !== undefined) {
            return dgram_1.default.createSocket({ type: 'udp6' });
        }
        else if (this.options.hosts.ip4 !== undefined) {
            return dgram_1.default.createSocket({ type: 'udp4' });
        }
        else if (this.options.hosts.ip6 !== undefined) {
            return dgram_1.default.createSocket({ type: 'udp6', ipv6Only: true });
        }
    }
    async startServer() {
        return new Promise(async (resolve, reject) => {
            this.socket = this.getSocket();
            this.socket.on('message', (msg, rinfo) => {
                const req = stun.createBlank();
                // if msg is valid STUN message
                if (req.loadBuffer(msg)) {
                    // if STUN message is BINDING_REQUEST and valid content
                    if (req.isBindingRequest({ fingerprint: true })) {
                        const res = req.createBindingResponse(true).setXorMappedAddressAttribute(rinfo).setFingerprintAttribute();
                        this.socket.send(res.toBuffer(), rinfo.port, rinfo.address);
                    }
                }
            });
            resolve(bindSocketToPort(this.socket));
        });
    }
    async stopServer() {
        if (this.socket) {
            await releaseSocketFromPort(this.socket);
        }
    }
}
exports.Stun = Stun;
function releaseSocketFromPort(socket) {
    return new Promise((resolve, reject) => {
        const onClose = () => {
            socket.removeListener('error', onError);
            setImmediate(resolve);
        };
        const onError = (err) => {
            socket.removeListener('close', onClose);
            reject(err);
        };
        socket.once('error', onError);
        socket.once('close', onClose);
        socket.close();
    });
}
function bindSocketToPort(socket, port = constants_1.DEFAULT_STUN_PORT) {
    return new Promise((resolve, reject) => {
        const onListening = () => {
            socket.removeListener('error', onError);
            resolve();
        };
        const onError = (err) => {
            socket.removeListener('listening', onListening);
            reject(err);
        };
        socket.once('error', onError);
        socket.once('listening', onListening);
        socket.bind(port);
    });
}
//# sourceMappingURL=stun.js.map