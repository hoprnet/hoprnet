/**
 * Logger Utility
 *
 * @author Patrice Juergens <pj@validitylabs.org>
 * @TODO: implement file logger (https://www.npmjs.com/package/winston)
 *
 * Special thanks to Andrianov Konstantsin (https://github.com/kandrianov)
 * Original source published under MIT license (https://github.com/etherisc/flightDelay/blob/master/util/logger.js)
 */

import path from 'path';
import chalk from 'chalk';

chalk.enabled = true;

/* Log levels */
const levels = {
    trace: {
        code: 0,
        time: chalk.gray,
        level: {
            decor: null,
            title: 'TRACE'
        },
        name: chalk.magenta,
        msg: chalk.white
    },
    debug: {
        code: 1,
        time: chalk.gray,
        level: {
            decor: chalk.blue,
            title: 'DEBUG'
        },
        name: chalk.magenta,
        msg: chalk.blue
    },
    info: {
        code: 2,
        time: chalk.gray,
        level: {
            decor: chalk.green.bold,
            title: 'INFO '
        },
        name: chalk.magenta,
        msg: chalk.cyan
    },
    warn: {
        code: 3,
        time: chalk.gray,
        level: {
            decor: chalk.black.bgYellow.bold,
            title: 'WARN'
        },
        name: chalk.magenta,
        msg: chalk.yellow
    },
    error: {
        code: 4,
        time: chalk.gray,
        level: {
            decor: chalk.bgRed.bold,
            title: 'ERROR'
        },
        name: chalk.magenta,
        msg: chalk.red
    }
};

let logLevel;

if (process.env.LOG_LEVEL) {
    if (Object.keys(levels).includes(process.env.LOG_LEVEL)) {
        logLevel = process.env.LOG_LEVEL;
    } else {
        console.log('Wrong LOG_LEVEL! Set to default: info');
        logLevel = 'info';
    }
} else {
    logLevel = 'info';
}

let filename = path.basename(module.parent.filename, '.js');

if (filename === 'index') {
    filename = path.basename(path.dirname(module.parent.filename));
}

/**
 * @returns {string} message
 */
const getTime = () => {
    const now   = new Date();
    const padL  = (num) => {
        const norm = Math.abs(Math.floor(num));
        return (norm < 10 ? '0' : '') + norm;
    };

    const padR  = (num) => {
        const len = String(num).length;
        return num + (len === 3 ? '' : len === 2 ? '0' : '00');
    };

    return `${padL(now.getHours())}:${padL(now.getMinutes())}:${padL(now.getSeconds())}.${padR(now.getMilliseconds())}`;
};

/**
 * @param {string} message Unformatted message
 * @returns {string} Formatted message
 */
const colorAddress = (message) => {
    const regex = /0x[A-Xa-x0-9]{0,}/;

    if (regex.test(message)) {
        return message.replace(new RegExp(regex), chalk.yellow(message.match(regex)[0]));
    }

    return message;
};

/**
 * --> time level [ name ] msg
 *
 * @param {*} lvl Level
 * @param {*} data Data
 * @returns {void}
 */
const log = (lvl, data) => {
    const message = (data !== null && typeof data === 'object') ? '[obj]' : data;

    if (message === 'eth_getFilterChanges') {
        return;
    }

    const time  = levels[lvl].time ? levels[lvl].time(getTime()) : getTime();
    const level = levels[lvl].level.decor ?
        levels[lvl].level.decor(levels[lvl].level.title) :
        levels[lvl].level.title;

    const name  = levels[lvl].name  ? levels[lvl].name(`[ ${filename} ]`)       : `[ ${filename} ]`;
    const msg   = levels[lvl].msg   ? levels[lvl].msg(colorAddress(message))    : message;

    if (message === '[obj]') {
        if (data.stack) {
            // Error
            console.log(`${time} ${level} ${name} ${chalk.red(data.message)}`);
            console.log(chalk.red.bold(`Type: ${data.name}`));

            if (data.code) {
                console.log(chalk.red.bold(`Code: ${data.code}`));
            }

            if (data.signal) {
                console.log(chalk.bgRed.bold(`Signal: ${data.signal}`));
            }

            console.log(chalk.red.bold('Stack:'));

            const stack = data.stack.split('\n');
            stack.shift();

            console.log(chalk.red.bold(stack.join('\n')));
        } else {
            // Other obj
            console.log(`${time} ${level} ${name} ${msg}`);
            console.log(data);
        }
    } else {
        // Simple string
        console.log(`${time} ${level} ${name} ${msg}`);
    }
};

const logger = {
    trace: (data) => {
        if (levels[logLevel].code > levels.trace.code) {
            return;
        }
        log('trace', data);
    },
    debug: (data) => {
        if (levels[logLevel].code > levels.debug.code) {
            return;
        }
        log('debug', data);
    },
    info: (data) => {
        if (levels[logLevel].code > levels.info.code) {
            return;
        }
        log('info', data);
    },
    warn: (data) => {
        if (levels[logLevel].code > levels.warn.code) {
            return;
        }
        log('warn', data);
    },
    error: (data) => {
        log('error', data);
    }
};

export {logger};
