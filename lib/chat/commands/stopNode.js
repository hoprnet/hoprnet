"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
class StopNode {
    constructor(node) {
        this.node = node;
    }
    /**
     * Stops the node and kills the process in case it does not quit by itself.
     */
    async execute() {
        const timeout = setTimeout(() => {
            console.log(`Ungracefully stopping node after timeout.`);
            process.exit(0);
        }, 10 * 1000);
        try {
            await this.node.stop();
            clearTimeout(timeout);
            process.exit(0);
        }
        catch (err) {
            console.log(chalk_1.default.red(err.message));
        }
    }
    complete(line, cb) {
        cb(undefined, [[''], line]);
    }
}
exports.default = StopNode;
//# sourceMappingURL=stopNode.js.map