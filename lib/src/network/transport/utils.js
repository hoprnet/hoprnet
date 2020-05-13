"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.isAnyAddr = exports.getMultiaddrs = exports.multiaddrToNetConfig = void 0;
const multiaddr_1 = __importDefault(require("multiaddr"));
const os_1 = __importDefault(require("os"));
const path_1 = require("path");
const ProtoFamily = { ip4: 'IPv4', ip6: 'IPv6' };
function multiaddrToNetConfig(addr) {
    const listenPath = addr.getPath();
    // unix socket listening
    if (listenPath) {
        return path_1.resolve(listenPath);
    }
    // tcp listening
    return addr.toOptions();
}
exports.multiaddrToNetConfig = multiaddrToNetConfig;
function getMultiaddrs(proto, ip, port) {
    const toMa = (ip) => multiaddr_1.default(`/${proto}/${ip}/tcp/${port}`);
    return (isAnyAddr(ip) ? getNetworkAddrs(ProtoFamily[proto]) : [ip]).map(toMa);
}
exports.getMultiaddrs = getMultiaddrs;
function isAnyAddr(ip) {
    return ['0.0.0.0', '::'].includes(ip);
}
exports.isAnyAddr = isAnyAddr;
function getNetworkAddrs(family) {
    let interfaces = os_1.default.networkInterfaces();
    let addresses = [];
    for (let iface of Object.values(interfaces)) {
        for (let netAddr of iface) {
            if (netAddr.family === family) {
                addresses.push(netAddr.address);
            }
        }
    }
    return addresses;
}
//# sourceMappingURL=utils.js.map