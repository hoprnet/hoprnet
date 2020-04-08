"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const path_1 = __importDefault(require("path"));
const fs_1 = __importDefault(require("fs"));
const ts_generator_1 = require("ts-generator");
const TypeChain_1 = require("typechain/dist/TypeChain");
async function main() {
    const root = path_1.default.join(__dirname, '..', '..');
    const asRepo = path_1.default.join(root, 'node_modules/@hoprnet/hopr-ethereum/build/extracted/abis');
    const asLib = path_1.default.join(root, '../../../node_modules/@hoprnet/hopr-ethereum/build/extracted/abis');
    const isRepo = fs_1.default.existsSync(asRepo);
    let isLib = false;
    if (!isRepo) {
        isLib = fs_1.default.existsSync(asLib);
    }
    if (!isRepo && !isLib) {
        throw Error("`hopr-ethereum` repo wasn't found");
    }
    await ts_generator_1.tsGenerator({ cwd: root }, new TypeChain_1.TypeChain({
        cwd: root,
        rawConfig: {
            files: `${isRepo ? asRepo : asLib}/*.json`,
            outDir: './src/tsc/web3',
            target: 'web3-v1'
        }
    }));
}
main().catch(console.error);
//# sourceMappingURL=typechain.js.map