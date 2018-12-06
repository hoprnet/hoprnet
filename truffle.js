/**
 * Truffle configuration
 */

const cnf               = require('./config/networks.json');
const HDWalletProvider  = require('truffle-hdwallet-provider');

require('babel-register');
require('babel-polyfill');

const network   = process.env.NETWORK;
let secrets     = '';

if (network === 'rinkebyInfura') {
    secrets = require('./config/.secrets.json');
}

const path      = require('path');
const basePath  = process.cwd();

const buildDir          = path.join(basePath, 'build');
const buildDirContracts = path.join(basePath, 'build/contracts');
const srcDir            = path.join(basePath, 'contracts');
const testDir           = path.join(basePath, 'test/contracts');
const migrationsDir     = path.join(basePath, 'migrations/contracts');

module.exports = {
    mocha: {
        useColors: true // disable bottom for testing dev/troubleshooting
        // reporter: 'eth-gas-reporter',
        // reporterOptions: {
        //     currency: 'CHF',
        //     gasPrice: cnf.networks.develop.gasPrice
        // }
    },
    solc: {
        optimizer: {
            enabled:    true,
            runs:       200
        }
    },
    networks: {
        develop: {
            host:       cnf.networks.develop.host,
            port:       cnf.networks.develop.port,
            network_id: cnf.networks.develop.chainId, // eslint-disable-line
            gas:        cnf.networks.develop.gas,
            gasPrice:   cnf.networks.develop.gasPrice
        },
        coverage: {
            host:       cnf.networks.coverage.host,
            network_id: cnf.networks.coverage.chainId, // eslint-disable-line
            port:       cnf.networks.coverage.port,
            gas:        cnf.networks.coverage.gas,
            gasPrice:   cnf.networks.coverage.gasPrice
        },
        rinkebyInfura:  getRinkebyConfig()
    },
    build_directory:            buildDir,            // eslint-disable-line
    contracts_build_directory:  buildDirContracts,   // eslint-disable-line
    migrations_directory:       migrationsDir,       // eslint-disable-line
    contracts_directory:        srcDir,              // eslint-disable-line
    test_directory:             testDir              // eslint-disable-line
};

function getRinkebyConfig() {
    let rinkebyProvider = '';

    if (network === 'rinkebyInfura') {
        rinkebyProvider = new HDWalletProvider(secrets.rinkeby.mnemonic, secrets.rinkeby.host);

        return {
            network_id: cnf.networks.rinkeby.chainId, // eslint-disable-line
            provider:   rinkebyProvider,
            from:       rinkebyProvider.getAddress(),
            gas:        cnf.networks.rinkeby.gas,
            gasPrice:   cnf.networks.rinkeby.gasPrice
        };
    }
}
