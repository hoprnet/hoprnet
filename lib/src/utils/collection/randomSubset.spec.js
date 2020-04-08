"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const randomSubset_1 = require("./randomSubset");
describe('testing random subset', function () {
    it('should return a subset with a filter function', function () {
        assert_1.default.deepEqual(randomSubset_1.randomSubset([1], 1), [1]);
        assert_1.default.deepEqual(randomSubset_1.randomSubset([1, 2, 3], 3).sort(), [1, 2, 3]);
        let array = [];
        for (let i = 0; i < 30; i++) {
            array.push(i);
        }
        let result = randomSubset_1.randomSubset(array, 10, (value) => value % 2 == 0);
        assert_1.default(result.length == 10);
        assert_1.default(result.every(value => value % 2 == 0));
        let set = new Set();
        array.forEach(value => {
            assert_1.default(0 <= value && value < 30);
            assert_1.default(!set.has(value));
            set.add(value);
        });
    });
    it('should return a subset', function () {
        assert_1.default.deepEqual(randomSubset_1.randomSubset([1, 2], 1, (value) => value == 1), [1]);
        assert_1.default.deepEqual(randomSubset_1.randomSubset([1, 2, 3], 3, (value) => [1, 2, 3].includes(value)).sort(), [1, 2, 3]);
        let array = [];
        for (let i = 0; i < 30; i++) {
            array.push(i);
        }
        let result = randomSubset_1.randomSubset(array, 10);
        assert_1.default(result.length == 10);
        let set = new Set();
        array.forEach(value => {
            assert_1.default(0 <= value && value < 30);
            assert_1.default(!set.has(value));
            set.add(value);
        });
    });
});
