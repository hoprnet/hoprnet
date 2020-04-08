"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const peer_info_1 = __importDefault(require("peer-info"));
const peer_id_1 = __importDefault(require("peer-id"));
const _1 = require(".");
describe('test crawl response generation', function () {
    it('should create a response', async function () {
        const failingResponse = new _1.CrawlResponse(undefined, {
            status: _1.CrawlStatus.FAIL
        });
        assert_1.default(failingResponse.status == _1.CrawlStatus.FAIL, 'Check status');
        assert_1.default.throws(() => new _1.CrawlResponse(undefined, {
            status: _1.CrawlStatus.OK
        }), `Should not create successful crawl responses without peerInfos.`);
        assert_1.default(new _1.CrawlResponse(failingResponse).status == _1.CrawlStatus.FAIL, 'Check status after parsing.');
        const peerInfos = [await peer_info_1.default.create(await peer_id_1.default.create({ keyType: 'secp256k1' }))];
        const successfulResponse = new _1.CrawlResponse(undefined, {
            status: _1.CrawlStatus.OK,
            peerInfos
        });
        assert_1.default(successfulResponse.status == _1.CrawlStatus.OK && (await successfulResponse.peerInfos)[0].id.toB58String() == peerInfos[0].id.toB58String(), 'Check status & peerInfo');
        assert_1.default((await successfulResponse.peerInfos)[0].id.toB58String() == peerInfos[0].id.toB58String(), 'Check peerInfo after parsing');
        peerInfos.push(await peer_info_1.default.create(await peer_id_1.default.create({ keyType: 'secp256k1' })));
        const secondSuccessfulResponse = new _1.CrawlResponse(undefined, {
            status: _1.CrawlStatus.OK,
            peerInfos
        });
        assert_1.default((await secondSuccessfulResponse.peerInfos).every((peerInfo, index) => peerInfos[index].id.toB58String() == peerInfo.id.toB58String()), 'Check multiple peerInfos');
    });
});
