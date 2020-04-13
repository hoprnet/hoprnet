"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const web3_1 = __importDefault(require("web3"));
const hopr_demo_seeds_1 = require("@hoprnet/hopr-demo-seeds");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const HoprToken_json_1 = __importDefault(require("@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json"));
const utils_1 = require("../utils");
const types_1 = require("../types");
const config_1 = require("../config");
const ACCOUNTS = [].concat(config_1.DEMO_ACCOUNTS, hopr_demo_seeds_1.BOOTSTRAP_SEEDS);
const AMOUNT = web3_1.default.utils.toWei('100', 'ether');
const privKeyToAddress = async (privKey) => {
    return utils_1.privKeyToPubKey(hopr_utils_1.stringToU8a(privKey))
        .then(utils_1.pubKeyToAccountId)
        .then(address => new types_1.AccountId(address).toHex());
};
async function main() {
    const web3 = new web3_1.default(config_1.DEFAULT_URI);
    const hoprToken = new web3.eth.Contract(HoprToken_json_1.default, config_1.TOKEN_ADDRESSES.private);
    const owner = await privKeyToAddress(config_1.FUND_ACCOUNT_PRIVATE_KEY);
    for (const privKey of ACCOUNTS) {
        const address = await privKeyToAddress(privKey);
        await hoprToken.methods.mint(address, AMOUNT).send({
            from: owner,
            gas: 200e3
        });
        console.log(`funded ${address}`);
    }
    // TODO: check if this is needed
    process.exit();
}
main().catch(console.error);
