import type {
  HardhatRuntimeEnvironment,
  SolcUserConfig,
  HardhatUserConfig,
  NetworksUserConfig,
  NetworkUserConfig,
  HttpNetworkUserConfig,
  HardhatNetworkUserConfig
} from 'hardhat/types'
// load env variables
require('dotenv').config()
// adds Ethereum-specific capabilities to the Chai assertion library
import '@nomicfoundation/hardhat-chai-matchers'
// load hardhat plugins
import '@nomiclabs/hardhat-ethers'
import '@nomiclabs/hardhat-solhint'
import 'hardhat-deploy'
import 'hardhat-gas-reporter'
import '@nomiclabs/hardhat-etherscan'
import 'solidity-coverage'
import '@typechain/hardhat'
import faucet, { type FaucetCLIOPts } from './tasks/faucet'
import parallelTest, { type ParallelTestCLIOpts } from './tasks/parallelTest'
import register, { type RegisterOpts } from './tasks/register'
import selfRegister, { type SelfRegisterOpts } from './tasks/selfRegister'
import requestTestTokens, { type RequestTestTokensOpts } from './tasks/requestTestTokens'
import disableAutoMine from './tasks/disableAutoMine'
import getAccounts from './tasks/getAccounts'
import type { NetworkOptions } from './types'

// Copied from `utils` to prevent from ESM import issues
// ESM support requires code changes within `hardhat-core`
/**
 *
 * @param input a string containing templated references to environment variables e.g. 'foo ${bar}'
 * @param vars a key-value vars storage object, e.g. { 'bar': 'bar_value' }
 * @returns a string with variables resolved to the actual values
 */
export function expandVars(input: string, vars: { [key: string]: any }) {
  return input.replace(/\$\{(.*)\}/g, (_, varName) => {
    if (!(varName in vars)) {
      throw new Error(`failed to expand vars in string '${input}', var ${varName} not defined`)
    }
    return vars[varName]
  })
}

// rest
import { task, types, extendEnvironment, subtask } from 'hardhat/config'
import { writeFileSync, realpathSync } from 'fs'
import { TASK_TEST_SETUP_TEST_ENVIRONMENT } from 'hardhat/builtin-tasks/task-names'
import { HARDHAT_NETWORK_NAME } from 'hardhat/plugins'
import stake, { StakeOpts } from './tasks/stake'
import { NetworkRegistryNftRank, MIN_STAKE } from './utils/constants'
import type { BigNumber } from 'ethers'

const { DEPLOYER_WALLET_PRIVATE_KEY, ETHERSCAN_KEY, BLOCKSCOUT_KEY, HOPR_ENVIRONMENT_ID, HOPR_HARDHAT_TAG } =
  process.env

const PROTOCOL_CONFIG = require('../core/protocol-config.json')

function ethereumUnitToNumberString(fee: string, hre: HardhatRuntimeEnvironment): BigNumber {
  const parsedGasPrice = fee.split(' ')

  let gasPrice: BigNumber
  if (parsedGasPrice.length > 1) {
    gasPrice = hre.ethers.BigNumber.from(hre.ethers.utils.parseUnits(parsedGasPrice[0], parsedGasPrice[1]).toString())
  } else {
    gasPrice = hre.ethers.BigNumber.from(parsedGasPrice[0])
  }

  return gasPrice
}

extendEnvironment((hre: HardhatRuntimeEnvironment) => {
  hre.environment = HOPR_ENVIRONMENT_ID as string
  hre.maxFeePerGas = ethereumUnitToNumberString(
    (PROTOCOL_CONFIG.networks[hre.network.name] as NetworkOptions).max_fee_per_gas,
    hre
  )
  hre.maxPriorityFeePerGas = ethereumUnitToNumberString(
    (PROTOCOL_CONFIG.networks[hre.network.name] as NetworkOptions).max_priority_fee_per_gas,
    hre
  )
})

// For reference on how the configuration is structured refer to:
//
// https://hardhat.org/hardhat-network/reference/#config
// https://github.com/wighawag/hardhat-deploy/blob/master/README.md
function networkToHardhatNetwork(name: String, input: NetworkOptions): NetworkUserConfig {
  let cfg: NetworkUserConfig = {
    chainId: input.chain_id,
    live: input.live,
    tags: input.tags,
    // used by hardhat-deploy
    saveDeployments: true
  }

  if (name !== 'hardhat') {
    try {
      ;(cfg as HttpNetworkUserConfig).url = expandVars(input.default_provider, process.env)
    } catch (_) {
      ;(cfg as HttpNetworkUserConfig).url = 'invalid_url'
    }
  } else {
    ;(cfg as HardhatNetworkUserConfig).initialDate = '2021-07-26'
  }

  if (input.live) {
    cfg.accounts = DEPLOYER_WALLET_PRIVATE_KEY ? [DEPLOYER_WALLET_PRIVATE_KEY] : []
    cfg.companionNetworks = {}
  }

  // we enable auto-mine only in development networks
  if (HOPR_HARDHAT_TAG) {
    cfg.tags = [HOPR_HARDHAT_TAG]
  }
  if (cfg.tags && cfg.tags.indexOf('development') >= 0) {
    ;(cfg as HardhatNetworkUserConfig).mining = {
      // Disabled using hardhat-specific RPC call after deployment
      auto: true, // every transaction will trigger a new block (without this deployments fail)
      interval: [1000, 3000] // mine new block every 1 - 3s
    }
  }
  if (input.etherscan_api_url) {
    ;(cfg as HardhatNetworkUserConfig).verify = {
      etherscan: {
        apiUrl: input.etherscan_api_url
      }
    }
  }
  return cfg
}

const networks: NetworksUserConfig = {}

for (const [networkId, network] of Object.entries<NetworkOptions>(PROTOCOL_CONFIG.networks)) {
  if (
    PROTOCOL_CONFIG.environments[HOPR_ENVIRONMENT_ID] &&
    PROTOCOL_CONFIG.environments[HOPR_ENVIRONMENT_ID].network_id === networkId
  ) {
    network['tags'] = [
      // always insert 'environment_type' as a tag so we know this info during smart contract deployment
      PROTOCOL_CONFIG.environments[HOPR_ENVIRONMENT_ID].environment_type,
      ...PROTOCOL_CONFIG.environments[HOPR_ENVIRONMENT_ID].tags
    ]
  }
  // environment could be undefined at this point
  const hardhatNetwork = networkToHardhatNetwork(networkId, network)
  networks[networkId] = hardhatNetwork
}

const hardhatConfig: HardhatUserConfig = {
  networks,
  namedAccounts: {
    deployer: 0,
    admin: {
      default: 1,
      goerli: '0xA18732DC751BE0dB04157eb92C92BA9d0fC09FC5',
      xdai: '0xE9131488563776DE7FEa238d6112c5dA46be9a9F'
    },
    alice: {
      default: 2,
      goerli: '0x3dA21EB3D7d40fEA6bd78c627Cc9B1F59E7481E1',
      xdai: '0x3dA21EB3D7d40fEA6bd78c627Cc9B1F59E7481E1'
    }
  },
  solidity: {
    compilers: ['0.8.9', '0.6.6', '0.4.24'].map<SolcUserConfig>((version) => ({
      version,
      settings: {
        optimizer: {
          enabled: true,
          runs: 200
        },
        outputSelection: {
          '*': {
            '*': ['storageLayout']
          }
        }
      }
    }))
  },
  paths: {
    sources: './contracts',
    tests: './test',
    cache: './hardhat/cache',
    artifacts: './hardhat/artifacts',
    // used by hardhat-deploy
    deployments: `./deployments/${HOPR_ENVIRONMENT_ID}`
  },
  typechain: {
    outDir: './src/types',
    target: 'ethers-v5'
  },
  gasReporter: {
    currency: 'USD',
    excludeContracts: ['mocks', 'utils/console.sol']
  },
  etherscan: {
    apiKey: {
      mainnet: ETHERSCAN_KEY,
      goerli: ETHERSCAN_KEY,
      xdai: BLOCKSCOUT_KEY
    }
  }
}

const DEFAULT_IDENTITY_DIRECTORY = '/tmp'
const DEFAULT_FUND_AMOUNT = '1'

task<FaucetCLIOPts>('faucet', 'Faucets a local development HOPR node account with ETH and HOPR tokens', faucet)
  .addOptionalParam<string>('address', 'destination address which should receive funds', undefined, types.string)
  .addOptionalParam<string>('amount', 'Amount of HOPR to fund', DEFAULT_FUND_AMOUNT, types.string)
  .addFlag('useLocalIdentities', `Fund all identities stored in identity directory`)
  .addOptionalParam<string>(
    'password',
    `Password to decrypt identities stored in identity directory`,
    undefined,
    types.string
  )
  .addOptionalParam<string>(
    'identityDirectory',
    `Overwrite default identity directory, default ['/tmp']`,
    DEFAULT_IDENTITY_DIRECTORY,
    types.string
  )
  .addOptionalParam<string>('identityPrefix', `only use identity files with prefix`, undefined, types.string)

task('accounts', 'View unlocked accounts', getAccounts)

task('disable-automine', 'Used by E2E tests to disable auto-mining once setup is done', disableAutoMine)

task<RegisterOpts>(
  'register',
  "Used by our E2E tests to interact with 'HoprNetworkRegistry' and 'HoprNetworkRegistryProxy'.",
  register
)
  .addParam<RegisterOpts['task']>('task', 'The task to run', undefined, types.string)
  .addOptionalParam<string>('nativeAddresses', 'A list of native addresses', undefined, types.string)
  .addOptionalParam<string>('peerIds', 'A list of comma-seperated peerIds', undefined, types.string)
  .addOptionalParam<string>('privatekey', 'Private key of the signer', undefined, types.string)

task<SelfRegisterOpts>(
  'register:self',
  "Used by our E2E tests to interact with 'HoprNetworkRegistry' and 'HoprNetworkRegistryProxy'.",
  selfRegister
)
  .addParam<SelfRegisterOpts['task']>('task', 'The task to run', undefined, types.string)
  .addParam<string>('peerIds', 'A list of comma-seperated peerIds', undefined, types.string)
  .addOptionalParam<string>('privatekey', 'Private key of the signer', undefined, types.string)

task<RequestTestTokensOpts>(
  'request-test-tokens',
  'Request test tokens ("Network_registry NFT" or "test wxHOPR (HOPR)") for a staker',
  requestTestTokens
)
  .addParam<RequestTestTokensOpts['type']>('type', 'Token type to request', undefined, types.string)
  .addOptionalParam<string>(
    'amount',
    'target txHOPR token amount (in wei) to request for',
    MIN_STAKE.toString(),
    types.string
  )
  .addParam<string>('recipient', 'Address of the NFT recipient', undefined, types.string)
  .addOptionalParam<NetworkRegistryNftRank>('nftRank', 'Network_registry NFT rank ("developer" or "community")', undefined, types.string)
  .addOptionalParam<string>('privatekey', 'Private key of the current owner of NFTs', undefined, types.string)

task<StakeOpts>('stake', 'Used by CI tests to stake tokens to the running staking program.', stake)
  .addParam<StakeOpts['type']>('type', 'Token type to stake', undefined, types.string)
  .addOptionalParam<string>(
    'amount',
    'target txHOPR token amount (in wei) that will be staked',
    MIN_STAKE.toString(),
    types.string
  )
  .addOptionalParam<NetworkRegistryNftRank>('nftRank', 'Network_registry NFT rank ("developer" or "community")', undefined, types.string)
  .addOptionalParam<string>('privatekey', 'Private key of the signer', undefined, types.string)

function getSortedFiles(dependenciesGraph) {
  const tsort = require('tsort')
  const graph = tsort()

  const filesMap = {}
  const resolvedFiles = dependenciesGraph.getResolvedFiles()
  resolvedFiles.forEach((f) => (filesMap[f.sourceName] = f))

  for (const [from, deps] of dependenciesGraph.entries()) {
    for (const to of deps) {
      graph.add(to.sourceName, from.sourceName)
    }
  }

  const topologicalSortedNames = graph.sort()

  // If an entry has no dependency it won't be included in the graph, so we
  // add them and then dedup the array
  const withEntries = topologicalSortedNames.concat(resolvedFiles.map((f) => f.sourceName))

  const sortedNames = [...new Set(withEntries)]
  return sortedNames.map((n: number) => filesMap[n])
}

function getFileWithoutImports(resolvedFile) {
  const IMPORT_SOLIDITY_REGEX = /^\s*import(\s+)[\s\S]*?;\s*$/gm

  return resolvedFile.content.rawContent.replace(IMPORT_SOLIDITY_REGEX, '').trim()
}

subtask('flat:get-flattened-sources', 'Returns all contracts and their dependencies flattened')
  .addOptionalParam('files', undefined, undefined, types.any)
  .addOptionalParam('output', undefined, undefined, types.string)
  .setAction(async ({ files, output }, { run }) => {
    const dependencyGraph = await run('flat:get-dependency-graph', { files })
    console.log(dependencyGraph)

    let flattened = ''

    if (dependencyGraph.getResolvedFiles().length === 0) {
      return flattened
    }

    const sortedFiles = getSortedFiles(dependencyGraph)

    let isFirst = true
    for (const file of sortedFiles) {
      if (!isFirst) {
        flattened += '\n'
      }
      flattened += `// File ${file.getVersionedName()}\n`
      flattened += `${getFileWithoutImports(file)}\n`

      isFirst = false
    }

    // Remove every line started with "// SPDX-License-Identifier:"
    flattened = flattened.replace(/SPDX-License-Identifier:/gm, 'License-Identifier:')

    flattened = `// SPDX-License-Identifier: MIXED\n\n${flattened}`

    // Remove every line started with "pragma experimental ABIEncoderV2;" except the first one
    flattened = flattened.replace(
      /pragma experimental ABIEncoderV2;\n/gm,
      (
        (i) => (m: string) =>
          !i++ ? m : ''
      )(0)
    )

    flattened = flattened.trim()
    if (output) {
      console.log('Writing to', output)
      writeFileSync(output, flattened)
      return ''
    }
    return flattened
  })

subtask('flat:get-dependency-graph')
  .addOptionalParam('files', undefined, undefined, types.any)
  .setAction(async ({ files }, { run }) => {
    const sourcePaths =
      files === undefined ? await run('compile:solidity:get-source-paths') : files.map((f: string) => realpathSync(f))

    const sourceNames = await run('compile:solidity:get-source-names', {
      sourcePaths
    })

    const dependencyGraph = await run('compile:solidity:get-dependency-graph', { sourceNames })

    return dependencyGraph
  })

task('flat', 'Flattens and prints contracts and their dependencies')
  .addOptionalVariadicPositionalParam('files', 'The files to flatten', undefined, types.inputFile)
  .addOptionalParam('output', 'Specify the output file', undefined, types.string)
  .setAction(async ({ files, output }, { run }) => {
    console.log(
      await run('flat:get-flattened-sources', {
        files,
        output
      })
    )
  })

subtask(TASK_TEST_SETUP_TEST_ENVIRONMENT, 'Setup test environment').setAction(async (_, { network }) => {
  if (network.name === HARDHAT_NETWORK_NAME) {
    await network.provider.send('hardhat_reset')
  }
})

subtask<ParallelTestCLIOpts>(
  'test:in-group:with-same-instance',
  'Put test files into groups that shares the same ganache instances',
  parallelTest
)

/**
 * parallelConfig.config contains an array of {testFiles: string[]} where the testFiles is an array
 * of relative paths of test files.
 * Test files in the same array share the same reset hardhat instance.
 * Test files that are in the default test path but not specified in the parallelConfig.config array
 * will be executed at the every end using a reset hardhat instance.
 */
task('test:in-group', 'Reset the hardhat node instances per testFiles array.').setAction(async ({}, { run }) => {
  const parallelConfig = {
    config: [
      {
        testFiles: ['stake/HoprBoost.test.ts']
      },
      {
        testFiles: ['stake/HoprStake.test.ts']
      },
      {
        testFiles: ['stake/HoprStake2.test.ts']
      },
      {
        testFiles: ['stake/HoprStakeSeason3.test.ts']
      },
      {
        testFiles: ['stake/HoprStakeSeason4.test.ts']
      },
      {
        testFiles: ['stake/HoprWhitehat.test.ts']
      }
    ]
  }
  await run('test:in-group:with-same-instance', parallelConfig)
})

export default hardhatConfig
