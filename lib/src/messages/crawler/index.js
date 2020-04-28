"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const resonse_1 = require("./resonse");
exports.CrawlResponse = resonse_1.CrawlResponse;
var CrawlStatus;
(function (CrawlStatus) {
    CrawlStatus[CrawlStatus["OK"] = 0] = "OK";
    CrawlStatus[CrawlStatus["FAIL"] = 1] = "FAIL";
})(CrawlStatus || (CrawlStatus = {}));
exports.CrawlStatus = CrawlStatus;
//# sourceMappingURL=index.js.map