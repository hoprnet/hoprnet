"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CrawlResponse = exports.CrawlStatus = void 0;
const resonse_1 = require("./resonse");
Object.defineProperty(exports, "CrawlResponse", { enumerable: true, get: function () { return resonse_1.CrawlResponse; } });
var CrawlStatus;
(function (CrawlStatus) {
    CrawlStatus[CrawlStatus["OK"] = 0] = "OK";
    CrawlStatus[CrawlStatus["FAIL"] = 1] = "FAIL";
})(CrawlStatus || (CrawlStatus = {}));
exports.CrawlStatus = CrawlStatus;
//# sourceMappingURL=index.js.map