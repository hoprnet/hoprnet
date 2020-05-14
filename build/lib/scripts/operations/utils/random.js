"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.isLocalNetwork = exports.getOperations = exports.getContractNames = exports.bash = exports.root = void 0;
const path_1 = __importDefault(require("path"));
const child_process_1 = require("child_process");
const truffle_networks_json_1 = __importDefault(require("../../../truffle-networks.json"));
exports.root = path_1.default.join(__dirname, '..', '..', '..');
exports.bash = (cmd) => {
    return new Promise((resolve, reject) => {
        try {
            const [first, ...rest] = cmd.split(' ');
            const child = child_process_1.spawn(first, rest, {
                cwd: exports.root,
            });
            child.stdout.setEncoding('utf8');
            child.stderr.setEncoding('utf8');
            child.stdout.on('data', console.log);
            child.stderr.on('data', console.error);
            child.on('exit', resolve);
            child.on('error', reject);
        }
        catch (err) {
            reject(err);
        }
    });
};
exports.getContractNames = () => {
    return ['HoprChannels', 'HoprMinter', 'HoprToken'];
};
exports.getOperations = () => {
    return ['patch', 'build', 'coverage', 'fund', 'migrate', 'network', 'test', 'verify'];
};
exports.isLocalNetwork = (network) => {
    return !!Object.entries(truffle_networks_json_1.default)
        .filter(([, config]) => config.network_id === '*')
        .find(([name]) => name === network);
};
