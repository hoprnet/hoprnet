import type {
  HardhatRuntimeEnvironment,
  HardhatConfig,
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
import 'solidity-coverage'
import '@typechain/hardhat'
import { utils } from 'ethers'
import faucet from './tasks/faucet'
import getAccounts from './tasks/getAccounts'

import { expandVars } from '@hoprnet/hopr-utils'

// rest
import { task, types, extendEnvironment, extendConfig, subtask } from 'hardhat/config'
import { writeFileSync, realpathSync } from 'fs'

const { DEPLOYER_WALLET_PRIVATE_KEY, ETHERSCAN_KEY, HOPR_ENVIRONMENT_ID, HOPR_HARDHAT_TAG } = process.env

const PROTOCOL_CONFIG = require('../core/protocol-config.json')

extendConfig((config: HardhatConfig) => {
  config.etherscan.apiKey = ETHERSCAN_KEY
})

extendEnvironment((hre: HardhatRuntimeEnvironment) => {
  hre.environment = HOPR_ENVIRONMENT_ID
})

// For reference on how the configuration is structured refer to:
//
// https://hardhat.org/hardhat-network/reference/#config
// https://github.com/wighawag/hardhat-deploy/blob/master/README.md
function networkToHardhatNetwork(name: String, input: any): NetworkUserConfig {
  let cfg: NetworkUserConfig = {
    chainId: input.chain_id,
    gasMultiplier: input.gas_multiplier,
    live: input.live,
    tags: input.tags,
    // used by hardhat-deploy
    saveDeployments: true
  }

  if (input.gas_price) {
    const parsedGasPrice = input.gas_price.split(' ')
    if (parsedGasPrice.length > 1) {
      cfg.gasPrice = Number(utils.parseUnits(parsedGasPrice[0], parsedGasPrice[1]))
    } else {
      cfg.gasPrice = Number(parsedGasPrice[0])
    }
  }

  if (name !== 'hardhat') {
    try {
      ;(cfg as HttpNetworkUserConfig).url = expandVars(input.default_provider, process.env)
    } catch (_) {
      ;(cfg as HttpNetworkUserConfig).url = 'invalid_url'
    }
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
      auto: true, // every transaction will trigger a new block (without this deployments fail)
      interval: [1000, 3000] // mine new block every 1 - 3s
    }
  }
  return cfg
}

const networks: NetworksUserConfig = {}

for (const [networkId, network] of Object.entries(PROTOCOL_CONFIG.networks)) {
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
    deployer: 0
  },
  solidity: {
    compilers: ['0.8.9', '0.6.6', '0.4.24'].map<SolcUserConfig>((version) => ({
      version,
      settings: {
        optimizer: {
          enabled: true,
          runs: 200
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
  }
}

const DEFAULT_IDENTITY_DIRECTORY = '/tmp'
const DEFAULT_FUND_AMOUNT = '1'

task('faucet', 'Faucets a local development HOPR node account with ETH and HOPR tokens', faucet)
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
        (i) => (m) =>
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

export default hardhatConfig
