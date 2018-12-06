'use strict';

/**
 * Main entry point for the task runner
 *
 * @author Patrice Juergens <pj@validitylabs.org>
 */

require('babel-polyfill');

const env   = process.env.NODE_ENV  || 'develop';
const task  = process.env.TASK;

if (env !== 'production') {
    require('babel-register');
}

const log = require('../lib/logger').logger;

if (!process.env.TASK) {
    log.error('No task passed!');
    process.exit(1);
}

log.info('======================================================');
// log.info('[ Args ]');
log.info('ENV\t: ' + env.toUpperCase());
log.info('Task\t: ' + task.toUpperCase());
// log.info('argv\t: ' + process.argv.toString());
log.info('======================================================');

require('./runner').run();
