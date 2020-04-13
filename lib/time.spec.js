"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const time_1 = require("./time");
describe('test time', function () {
    context('durations', function () {
        it('should be 1 second', function () {
            assert_1.default(time_1.durations.seconds(1) === 1e3, 'check durations.seconds');
        });
        it('should be 2 seconds', function () {
            assert_1.default(time_1.durations.seconds(2) === 2e3, 'check durations.seconds');
        });
        it('should be 1 minute', function () {
            assert_1.default(time_1.durations.minutes(1) === 1e3 * 60, 'check durations.minutes');
        });
        it('should be 2 minutes', function () {
            assert_1.default(time_1.durations.minutes(2) === 2e3 * 60, 'check durations.minutes');
        });
        it('should be 1 hour', function () {
            assert_1.default(time_1.durations.hours(1) === 1e3 * 60 * 60, 'check durations.hours');
        });
        it('should be 2 hours', function () {
            assert_1.default(time_1.durations.hours(2) === 2e3 * 60 * 60, 'check durations.hours');
        });
        it('should be 1 day', function () {
            assert_1.default(time_1.durations.days(1) === 1e3 * 60 * 60 * 24, 'check durations.days');
        });
        it('should be 2 days', function () {
            assert_1.default(time_1.durations.days(2) === 2e3 * 60 * 60 * 24, 'check durations.days');
        });
    });
});
