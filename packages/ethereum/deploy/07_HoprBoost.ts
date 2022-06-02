import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts } = hre

  const { deploy } = deployments
  const { deployer } = await getNamedAccounts() // Deployer is still the admin and minter

  await deploy('HoprBoost', {
    from: deployer,
    args: [deployer, ''],
    log: true
  })
}
main.tags = ['HoprBoost']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production || !!env.network.tags.staging

export default main
