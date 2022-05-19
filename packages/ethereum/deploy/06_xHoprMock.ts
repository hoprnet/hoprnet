import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'
import type { ERC677Mock } from '../src/types'

const MINTED_AMOUNT = '5000000'

const main: DeployFunction = async function ({ ethers, deployments, getNamedAccounts }: HardhatRuntimeEnvironment) {
  const { deploy } = deployments
  const { deployer, admin } = await getNamedAccounts()

  const xHoprContract = await deploy('xHoprMock', {
    contract: 'ERC677Mock',
    from: deployer,
    log: true
  })

  // mint xHOPR to admin
  const xhoprToken = (await ethers.getContractFactory('ERC677Mock')).attach(xHoprContract.address) as ERC677Mock
  await xhoprToken.batchMintInternal([admin], ethers.utils.parseUnits(MINTED_AMOUNT, 'ether'))
  console.log(`Admin gets minted ${MINTED_AMOUNT} xHOPR (mock) tokens`)
}
main.tags = ['xHoprMock']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production

export default main
