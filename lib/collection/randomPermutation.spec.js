"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const randomPermutation_1 = require("./randomPermutation");
describe('testing random permutation', function () {
    let ATTEMPTS = 2;
    it(`should apply a random permutation`, function () {
        for (let counter = 0; counter < ATTEMPTS; counter++) {
            let array = [];
            for (let i = 0; i < 30; i++) {
                array.push(i);
            }
            let length = array.length;
            randomPermutation_1.randomPermutation(array);
            assert_1.default(array.length == length);
            let set = new Set();
            array.forEach((value) => {
                assert_1.default(!set.has(value));
                set.add(value);
            });
        }
    });
});
