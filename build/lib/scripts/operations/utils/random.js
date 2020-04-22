"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const path_1 = __importDefault(require("path"));
const child_process_1 = require("child_process");
exports.root = path_1.default.join(__dirname, '..', '..', '..');
exports.bash = (cmd) => {
    return new Promise((resolve, reject) => {
        const [first, ...rest] = cmd.split(' ');
        const child = child_process_1.spawn(first, rest, {
            cwd: exports.root
        });
        child.stdout.setEncoding('utf8');
        child.stderr.setEncoding('utf8');
        child.stdout.on('data', console.log);
        child.stderr.on('data', console.error);
        child.on('close', resolve);
        child.on('exit', resolve);
        child.on('error', reject);
    });
};
