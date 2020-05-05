"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const perf_hooks_1 = require("perf_hooks");
function timer(fn) {
    const start = perf_hooks_1.performance.now();
    fn();
    const end = perf_hooks_1.performance.now() - start;
    return end;
}
exports.timer = timer;
exports.MAX_EXECUTION_TIME_FOR_CONCAT_IN_MS = 1;
