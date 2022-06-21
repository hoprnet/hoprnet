import { Signer, Wallet } from 'ethers'
import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'

export type StakeOpts = {
  amount: string // target amount in wei
  privatekey: string // private key of the caller
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

  let signer: Signer;
  if (!opts.privatekey) {
    signer = provider.getSigner()
  } else {
    signer = new Wallet(opts.privatekey, provider)
  }
  const signerAddress = await signer.getAddress()

  console.log("Signer Address", signerAddress)
  
  const hoprToken = (await ethers.getContractFactory('ERC677Mock')).connect(signer).attach(tokenContract.address)
  const hoprStake = (await ethers.getContractFactory('HoprStakeSeason3'))
    .connect(signer)
    .attach(stakingContract.address)

  const stakedAmount = (await hoprStake.stakedHoprTokens(signerAddress)).toString()
  if (ethers.BigNumber.from(stakedAmount).gte(ethers.BigNumber.from(opts.amount))) {
    console.log(`Account ${signerAddress} has staked enough.`)
    return
  }

  const amountToStake = ethers.BigNumber.from(opts.amount).sub(ethers.BigNumber.from(stakedAmount)).toString()

  try {
    await (await hoprToken.transferAndCall(stakingContract.address, amountToStake, ethers.constants.HashZero)).wait()
    console.log(`Stake ${opts.amount} tokens in the staking contract`)
  } catch (error) {
    console.error(`Cannot stake via task due to ${error}`)
  }
}

export default main
