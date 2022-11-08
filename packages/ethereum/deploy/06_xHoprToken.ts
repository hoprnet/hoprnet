import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { ERC677Mock } from '../src/types'

const MINTED_AMOUNT = '5000000'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, environment, maxFeePerGas, maxPriorityFeePerGas } = hre
  const { admin } = await getNamedAccounts()
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const deployOptions = {
    log: true
  }
  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    deployOptions['waitConfirmations'] = 2
  }

  console.log(`Deploying xHoprToken (mock) with account ${deployer.address}`)
  const xHoprContract = await deployments.deploy('xHoprToken', {
    contract: 'ERC677Mock',
    from: deployer.address,
    maxFeePerGas,
    maxPriorityFeePerGas,
    ...deployOptions
  })
  console.log(`xHoprToken (mock) deployed at ${xHoprContract.address}`)

  const xhoprToken = (await ethers.getContractFactory('ERC677Mock')).attach(xHoprContract.address) as ERC677Mock

  const amount = ethers.utils.parseUnits(MINTED_AMOUNT, 'ether')
  console.log(`Minting ${amount} xHoprToken (mock) tokens to account ${admin}`)
  const mintTx = await xhoprToken.batchMintInternal([admin], amount)

  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    console.log(`Wait for minting tx on chain`)
    await ethers.provider.waitForTransaction(mintTx.hash, 2)
  }

  console.log(`Minted ${amount} xHoprToken (mock) tokens to account ${admin}`)
}

main.tags = ['xHoprToken']
main.dependencies = ['preDeploy', 'HoprChannels']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production

export default main
