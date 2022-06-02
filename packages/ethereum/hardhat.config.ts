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
// load hardhat plugins
import '@nomiclabs/hardhat-ethers'
import '@nomiclabs/hardhat-solhint'
import '@nomiclabs/hardhat-waffle'
import 'hardhat-deploy'
import 'hardhat-gas-reporter'
import '@nomiclabs/hardhat-etherscan'
import 'solidity-coverage'
import '@typechain/hardhat'
import faucet, { type FaucetCLIOPts } from './tasks/faucet'
import parallelTest, { type ParallelTestCLIOpts } from './tasks/parallelTest'
import register, { type RegisterOpts } from './tasks/register'
import disableAutoMine from './tasks/disableAutoMine'
import getAccounts from './tasks/getAccounts'

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

// Copied from `core` to prevent from ESM import issues
// ESM support requires code changes within `hardhat-core`
type NetworkOptions = {
  id: string
  description: string
  chain_id: number // >= 0
  live: boolean
  hardhat_deploy_gas_price: string // Gas price as either a number string '11' or a value which should be converted like '1 gwei'. Used in hardhat-deploy plugin.
  default_provider: string // a valid HTTP url pointing at a RPC endpoint
  etherscan_api_url?: string // a valid HTTP url pointing at a RPC endpoint
  max_fee_per_gas: string // The absolute maximum you are willing to pay per unit of gas to get your transaction included in a block, e.g. '10 gwei'
  max_priority_fee_per_gas: string // Tips paid directly to miners, e.g. '2 gwei'
  native_token_name: string
  hopr_token_name: string
  tags: string[]
}
// rest
import { task, types, extendEnvironment, subtask } from 'hardhat/config'
import { writeFileSync, realpathSync } from 'fs'
import { TASK_TEST_SETUP_TEST_ENVIRONMENT } from 'hardhat/builtin-tasks/task-names'
import { HARDHAT_NETWORK_NAME } from 'hardhat/plugins'
import { TASK_DEPLOY_RUN_DEPLOY } from 'hardhat-deploy'

const { DEPLOYER_WALLET_PRIVATE_KEY, ETHERSCAN_KEY, HOPR_ENVIRONMENT_ID, HOPR_HARDHAT_TAG } = process.env

const PROTOCOL_CONFIG = require('../core/protocol-config.json')

extendEnvironment((hre: HardhatRuntimeEnvironment) => {
  hre.environment = HOPR_ENVIRONMENT_ID
})

// inlined from @hoprnet/hopr-utils to remove dependency on whole package
function expandVars(input: string, vars: { [key: string]: any }) {
  return input.replace(/\$\{(.*)\}/g, (_, varName) => {
    if (!(varName in vars)) {
      throw new Error(`failed to expand vars in string '${input}', var ${varName} not defined`)
    }
    return vars[varName]
  })
}


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
    network['tags'] = PROTOCOL_CONFIG.environments[HOPR_ENVIRONMENT_ID].tags
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
  verify: {
    etherscan: {
      apiKey: ETHERSCAN_KEY
    }
  }
}

const DEFAULT_IDENTITY_DIRECTORY = '/tmp'
const DEFAULT_FUND_AMOUNT = '1'

task<FaucetCLIOPts>('faucet', 'Faucets a local development HOPR node account with ETH and HOPR tokens', faucet)
  .addOptionalParam<string>('address', 'HoprToken address', undefined, types.string)
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
  "Used by our E2E tests to interact with 'HoprNetworkRegistry' and 'HoprDummyProxyForNetworkRegistry'.",
  register
)
  .addParam<RegisterOpts['task']>('task', 'The task to run', undefined, types.string)
  .addOptionalParam<string>('nativeAddresses', 'A list of native addresses', undefined, types.string)
  .addOptionalParam<string>('peerIds', 'A list of peerIds', undefined, types.string)

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

/**
 * Override https://github.com/wighawag/hardhat-deploy/blob/819df0fad56d75a5de5218c3307bec2093f8794c/src/index.ts#L396
 * in hardhat-deploy plugin, as it does not support EIP-1559
 */
subtask(TASK_DEPLOY_RUN_DEPLOY, 'Override the deploy task, with an explicit gas price.').setAction(
  async (taskArgs, { network, ethers }, runSuper) => {
    const protocolConfigNetwork = PROTOCOL_CONFIG.networks[network.name] ?? undefined
    if (!protocolConfigNetwork) {
      throw Error(
        'Cannot deploy with hardhat-deploy due to missing hardhat_deploy_gas_price field in protocol-config.json file'
      )
    }

    const hardhatDeployGasPrice = (protocolConfigNetwork as NetworkOptions).hardhat_deploy_gas_price
    const parsedGasPrice = hardhatDeployGasPrice.split(' ')

    // as in https://github.com/wighawag/hardhat-deploy/blob/819df0fad56d75a5de5218c3307bec2093f8794c/src/DeploymentsManager.ts#L974
    let gasPrice: string
    if (parsedGasPrice.length > 1) {
      gasPrice = ethers.utils.parseUnits(parsedGasPrice[0], parsedGasPrice[1]).toString()
    } else {
      gasPrice = parsedGasPrice[0]
    }

    console.log(`Deployment arguments are ${JSON.stringify({ ...taskArgs, gasprice: gasPrice }, null, 2)}`)

    try {
      await runSuper({ ...taskArgs, gasprice: gasPrice })
    } catch (error) {
      console.log(error)
      throw Error('Cannot override hardhat task TASK_DEPLOY_RUN_DEPLOY')
    }
  }
)
export default hardhatConfig
