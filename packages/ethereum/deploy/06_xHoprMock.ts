import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'
import type { ERC677Mock } from '../src/types'

const MINTED_AMOUNT = '5000000'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, environment } = hre
  const { admin } = await getNamedAccounts()
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const deployOptions = {
    log: true
  }
  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    deployOptions['waitConfirmations'] = 2
  }

  const xHoprContract = await deployments.deploy('xHoprMock', {
    contract: 'ERC677Mock',
    from: deployer.address,
    ...deployOptions
  })

  // mint xHOPR to admin
  const xhoprToken = (await ethers.getContractFactory('ERC677Mock')).attach(xHoprContract.address) as ERC677Mock
  const mintTx = await xhoprToken.batchMintInternal([admin], ethers.utils.parseUnits(MINTED_AMOUNT, 'ether'))

  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    await ethers.provider.waitForTransaction(mintTx.hash, 2)
  }

  console.log(`Admin minted ${MINTED_AMOUNT} xHOPR (mock) tokens`)
}

main.tags = ['xHoprMock']
main.dependencies = ['preDeploy', 'HoprChannels']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production

export default main
