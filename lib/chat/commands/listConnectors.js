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
const chalk_1 = __importDefault(require("chalk"));
const __1 = require("..");
class ListConnectors {
    /**
     * Check which connectors are present right now.
     * @notice triggered by the CLI
     */
    async execute() {
        let str = 'Available connectors:';
        let found = 0;
        const promises = [];
        for (let i = 0; i < __1.knownConnectors.length; i++) {
            promises.push(Promise.resolve().then(() => __importStar(require(__1.knownConnectors[i][0]))).then(() => {
                found++;
                str += `\n  ${chalk_1.default.yellow(__1.knownConnectors[i][0])} ${chalk_1.default.gray('=>')} ./hopr -n ${chalk_1.default.green(__1.knownConnectors[i][1])}`;
            }, () => { }));
        }
        await Promise.all(promises);
        if (found > 0) {
            console.log(str);
        }
        else {
            console.log(chalk_1.default.red(`Could not find any connectors. Please make sure there is one available in 'node_modules'!`));
        }
    }
    complete(line, cb) {
        cb(undefined, [[''], line]);
    }
}
exports.default = ListConnectors;
//# sourceMappingURL=listConnectors.js.map