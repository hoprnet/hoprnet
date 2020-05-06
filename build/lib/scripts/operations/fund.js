"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const web3_1 = __importDefault(require("web3"));
const HoprToken_json_1 = __importDefault(require("../../build/extracted/abis/HoprToken.json"));
const truffle_networks_json_1 = __importDefault(require("../../truffle-networks.json"));
const AMOUNT = web3_1.default.utils.toWei('1000000', 'ether');
exports.default = async () => {
    const web3 = new web3_1.default(`ws://${truffle_networks_json_1.default.development.host}:${truffle_networks_json_1.default.development.port}`);
    const hoprToken = new web3.eth.Contract(HoprToken_json_1.default, '0x66DB78F4ADD912a6Cb92b672Dfa09028ecc3085E');
    const accounts = await web3.eth.getAccounts();
    const owner = accounts[0];
    for (const account of accounts) {
        await hoprToken.methods.mint(account, AMOUNT, '0x00', '0x00').send({
            from: owner,
            gas: 200e3,
        });
        console.log(`funded ${account}`);
    }
    // @ts-ignore
    web3.currentProvider.disconnect();
};
