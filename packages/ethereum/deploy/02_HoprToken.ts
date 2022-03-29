import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { HoprToken } from '../src/types'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')

const main: DeployFunction = async function ({
  ethers,
  deployments,
  network,
  getNamedAccounts,
  environment
}: HardhatRuntimeEnvironment) {
  const environmentConfig = PROTOCOL_CONFIG.environments[environment]
  const mintedTokenReceiver = environmentConfig['minted_token_receiver_address']

  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const tokenContract = await deployments.deploy('HoprToken', {
    from: deployer.address,
    log: true
  })

  if (network.tags.testing || network.tags.development) {
    const hoprToken = (await ethers.getContractFactory('HoprToken')).attach(tokenContract.address) as HoprToken
    const MINTER_ROLE = await hoprToken.MINTER_ROLE()
    const isDeployerMinter = await hoprToken.hasRole(MINTER_ROLE, deployer.address)

    // on "testing" networks, we cannot wait 10 blocks as there is no auto-mine
    // on "development" networks, we must wait 10 blocks since hardhat is not aware of the txs
    if (!isDeployerMinter) {
      console.log('Granting MINTER role to', deployer.address)
      await hoprToken.grantRole(MINTER_ROLE, deployer.address)

      if (mintedTokenReceiver) {
        console.log('Minting tokens to', mintedTokenReceiver)
        await hoprToken.mint(
          mintedTokenReceiver,
          ethers.utils.parseEther('130000000'),
          ethers.constants.HashZero,
          ethers.constants.HashZero
        )
      }
    }
  }
}

// this smart contract should not be redeployed on a production network
main.skip = async (env) => !!env.network.tags.production
main.dependencies = ['preDeploy']
main.tags = ['HoprToken']

export default main
