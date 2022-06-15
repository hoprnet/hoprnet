import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'

async function main(
  _opts: {},
  { network, ethers }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
): Promise<void> {
  if (!network.tags.testing && !network.tags.development) {
    throw Error(`Auto-mining is only present in testing or development networks`)
  }

  const provider = new ethers.providers.JsonRpcProvider()
  // Use hardhat-specific EVM call to stop auto-mining in order
  // to get more accurate testing behavior
  await provider.send('evm_setAutomine', [false])
}

export default main
