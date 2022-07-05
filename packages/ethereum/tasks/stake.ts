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
  { ethers, deployments, environment, network }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  if (environment == undefined) {
    console.error(`HOPR_ENVIRONMENT_ID is not set. Run with "HOPR_ENVIRONMENT_ID=<environment> ..."`)
    process.exit(1)
  }

  const tokenContract = await deployments.get('xHoprMock')
  const stakingContract = await deployments.get('HoprStake')

  // we use a custom ethers provider here instead of the ethers object from the
  // hre which is managed by hardhat-ethers, because that one seems to
  // run its own in-memory hardhat instance, which is undesirable
  const provider = new ethers.providers.JsonRpcProvider(network.config.url)

  let signer: Signer
  if (!opts.privatekey) {
    signer = provider.getSigner()
  } else {
    signer = new Wallet(opts.privatekey, provider)
  }
  const signerAddress = await signer.getAddress()

  console.log('Signer Address', signerAddress)

  const hoprToken = (await ethers.getContractFactory('ERC677Mock')).connect(signer).attach(tokenContract.address)
  const hoprStake = (await ethers.getContractFactory('HoprStakeSeason3'))
    .connect(signer)
    .attach(stakingContract.address)

  const balanceNativeToken = await provider.getBalance(signerAddress)
  let balanceHoprToken: number
  try {
    balanceHoprToken = await hoprToken.balanceOf(signerAddress)
  } catch (_) {
    balanceHoprToken = 0
  }
  console.log(`Account ${signerAddress} has ${balanceHoprToken} HOPR tokens`)
  console.log(`Account ${signerAddress} has ${balanceNativeToken} native tokens`)

  if (balanceNativeToken <= 0) {
    console.log(`Account ${signerAddress} does not have enough native tokens to proceed`)
    process.exit(1)
  }

  let stakedAmount: number = 0
  try {
    stakedAmount = (await hoprStake.stakedHoprTokens(signerAddress)).toString()
    console.log(`Account ${signerAddress} has staked ${stakedAmount}.`)
  } catch (_) {
    console.log(`Account ${signerAddress} has not staked anything yet`)
  }

  if (ethers.BigNumber.from(stakedAmount).gte(ethers.BigNumber.from(opts.amount))) {
    console.log(`Account ${signerAddress} has staked enough.`)
    return
  }

  const amountToStake = ethers.BigNumber.from(opts.amount).sub(ethers.BigNumber.from(stakedAmount))
  console.log(`Account ${signerAddress} has to stake ${opts.amount} HOPR tokens`)

  if (amountToStake.gt(ethers.BigNumber.from(balanceHoprToken))) {
    console.log(`Account ${signerAddress} does not have enough HOPR tokens to proceed with staking`)
    process.exit(1)
  }

  try {
    await (await hoprToken.transferAndCall(stakingContract.address, amountToStake, ethers.constants.HashZero)).wait()
    console.log(`Account ${signerAddress} staked ${opts.amount} HOPR tokens successfully`)
  } catch (error) {
    console.error(`Staking HOPR tokens failed due to ${error}`)
  }
}

export default main
