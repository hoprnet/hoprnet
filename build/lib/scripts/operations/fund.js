"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const web3_1 = __importDefault(require("web3"));
const HoprToken_json_1 = __importDefault(require("../../build/extracted/abis/HoprToken.json"));
const AMOUNT = web3_1.default.utils.toWei('1000000', 'ether');
exports.default = async () => {
    const web3 = new web3_1.default(`ws://127.0.0.1:9545`);
    const hoprToken = new web3.eth.Contract(HoprToken_json_1.default, '0x302be990306f95a21905d411450e2466DC5DD927');
    const accounts = await web3.eth.getAccounts();
    const owner = accounts[0];
    for (const account of accounts) {
        await hoprToken.methods.mint(account, AMOUNT).send({
            from: owner,
            gas: 200e3
        });
        console.log(`funded ${account}`);
    }
    web3.currentProvider['disconnect']();
};
