"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const dotenv_1 = __importDefault(require("dotenv"));
dotenv_1.default.config();
const assert_1 = __importDefault(require("assert"));
const utils_1 = require("../../utils");
// @ts-ignore
const libp2p = require("libp2p");
// @ts-ignore
const TCP = require("libp2p-tcp");
// @ts-ignore
const MPLEX = require("libp2p-mplex");
// @ts-ignore
const SECIO = require("libp2p-secio");
const hopr_core_polkadot_1 = require("@hoprnet/hopr-core-polkadot");
const config = __importStar(require("@hoprnet/hopr-core-polkadot/lib/config"));
const _1 = require(".");
const utils_2 = require("../../utils");
const bn_js_1 = __importDefault(require("bn.js"));
async function generateNode(id) {
    const peerId = await utils_2.privKeyToPeerId(config.DEMO_ACCOUNTS[id]);
    const node = (await libp2p.create({
        peerInfo: await utils_1.getPeerInfo({ id, peerId }),
        modules: {
            transport: [TCP],
            streamMuxer: [MPLEX],
            connEncryption: [SECIO]
        }
    }));
    await node.start();
    node.paymentChannels = {
        types: hopr_core_polkadot_1.Types,
        self: {
            privateKey: peerId.privKey.marshal(),
            publicKey: peerId.pubKey.marshal()
        }
    };
    node.interactions = {
        payments: new _1.PaymentInteractions(node)
    };
    return node;
}
describe('test payment (channel) interactions', function () {
    it('should establish a connection and run through the opening sequence', async function () {
        const [Alice, Bob] = await Promise.all([generateNode(0), generateNode(1)]);
        const testArray = new Uint8Array(32).fill(0xff);
        const response = new Uint8Array(hopr_core_polkadot_1.Types.SignedChannel.SIZE).fill(0x00);
        Bob.paymentChannels = {
            channel: {
                handleOpeningRequest(_) {
                    return (source) => {
                        return (async function* () {
                            for await (const chunk of source) {
                                assert_1.default(chunk.length > 0, 'Should receive a message');
                                yield response.slice();
                            }
                        })();
                    };
                }
            }
        };
        assert_1.default((await Alice.interactions.payments.open.interact(Bob.peerInfo, {
            balance: new bn_js_1.default(123456),
            balance_a: new bn_js_1.default(123),
            toU8a: () => testArray
        })) != null, 'Should a receive a message from counterparty');
        await Promise.all([
            Alice.stop(),
            Bob.stop()
        ]);
    });
});
