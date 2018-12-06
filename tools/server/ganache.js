/**
 * Ganache RPC server
 *
 * @author Patrice Juergens <pj@validitylabs.org>
 * @see https://github.com/trufflesuite/ganache-cli/issues/257
 */

import {logger as log} from '../lib/logger';
import cnf from '../../config/networks.json';
import sh from 'shelljs';
import Ganache from 'ganache-core';
import Web3 from 'web3';
import net from 'net';
import prompt from 'prompt';

// Ensure a fresh DB folder is there
sh.rm('-fr', './db');
sh.mkdir('-p', './db');

const PORT = cnf.networks.develop.port;
const web3 = new Web3(new Web3.providers.HttpProvider('http://' + cnf.networks.develop.host + ':' + PORT));

async function isPortAvailable(port) {
    isPortAvailable.lastError = '';

    return new Promise((resolve) => {
        // if port is not a number or is not an integet or is out of range block
        if (isNaN(port) || port !== parseInt(port, 10) || port < 0 || port > 65536) {
            isPortAvailable.lastError = 'Ivalid input. Port must be an Integer number betwen 0 and 65536';
            resolve(false);
        }
        // do the test
        port = parseInt(port, 10);
        const tester = net.createServer()
            // catch errors, and resolve false
            .once('error', (err) => {
                isPortAvailable.lastError = err.code || err; // EADDRINUSE , EACCES ...
                resolve(false);
            })
            // return true if succed
            .once('listening', () => tester.once('close', () => resolve(true)).close())
            .listen(port);
    });
}

function startServer() {
    const config = {
        accounts: [
            {
                balance: '0xd3c21bcecceda0000000'
            },
            {
                balance: '0xd3c21bcecceda0000000'
            },
            {
                balance: '0xd3c21bcecceda0000000'
            },
            {
                balance: '0xd3c21bcecceda0000000'
            },
            {
                balance: '0xd3c21bcecceda0000000'
            },
            {
                balance: '0xd3c21bcecceda0000000'
            },
            {
                balance: '0xd3c21bcecceda0000000'
            },
            {
                balance: '0xd3c21bcecceda0000000'
            },
            {
                balance: '0xd3c21bcecceda0000000'
            },
            {
                balance: '0xd3c21bcecceda0000000'
            }
        ],
        mnemonic: cnf.networks.develop.mnemonic,
        port: PORT,
        locked: false,
        // debug: true,
        gasPrice: cnf.networks.develop.gasPrice,
        gasLimit: cnf.networks.develop.gas,
        network_id: cnf.networks.develop.chainId,
        db_path: './db/'
    };

    if (process.env.verbose) {
        config.logger = {
            log: log.info
        };
    }

    const ganache = Ganache.server(config);

    ganache.listen(PORT, async (err) => {
        if (err) {
            log.error(err);
        } else {
            const accounts = await web3.eth.getAccounts();

            if (process.env.verbose) {
                log.info('Ganache Server output:');
                log.info('Host\t: http://' + cnf.networks.develop.host + ':' + PORT);
                log.info('mnemonic\t: ' + config.mnemonic);
                log.info('accounts\t: ');

                accounts.forEach(async (account) => {
                    log.info(account + ' : ' + await web3.eth.getBalance(account));
                });
            }
        }
    });
}

(async () => {
    if (await isPortAvailable(PORT)) {
        startServer();
    } else {
        const schema = {
            properties: {
                kill: {
                    pattern: /^(yes|no)$/,
                    message: 'Only yes or no allowed',
                    description: 'yes / no',
                    default: 'no',
                    required: true
                }
            }
        };

        log.warn(`Port ${PORT} already in use!`);
        log.warn(`Kill REALLY ALL node processes with open port ${PORT}?`);

        prompt.start();

        prompt.get(schema, (err, result) => {
            if (err) {
                log.error(err);
                process.exit(1);
            } else {
                if (result.kill === 'yes') {
                    sh.exec(`lsof -i tcp:${PORT} | grep node | awk '{print $2}' | xargs kill -9`, (code, stdout) => {
                        log.info(stdout);
                    });
                } else {
                    log.info('Command aborted');
                    process.exit(0);
                }
            }
        });
    }
})();
