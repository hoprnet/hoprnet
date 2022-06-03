import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'

export type StakeOpts = {
  amount: string // amount in wei
}

/**
 * Let caller to stake HOPR tokens in the current staking program
 */
async function main(
  opts: StakeOpts,
  { ethers, deployments }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  const tokenContract = await deployments.get('xHoprMock')
  const stakingContract = await deployments.get('HoprStake')

  // we use a custom ethers provider here instead of the ethers object from the
  // hre which is managed by hardhat-ethers, because that one seems to
  // run its own in-memory hardhat instance, which is undesirable
  const provider = new ethers.providers.JsonRpcProvider()
  const signer = provider.getSigner()

  const hoprToken = (await ethers.getContractFactory('ERC677Mock')).connect(signer).attach(tokenContract.address)

  try {
    await (await hoprToken.transferAndCall(stakingContract.address, opts.amount, ethers.constants.HashZero)).wait()
    console.log(`Stake ${opts.amount} tokens in the staking contract`)
  } catch (error) {
    console.error(`Cannot stake via task due to ${error}`)
  }
}

export default main
