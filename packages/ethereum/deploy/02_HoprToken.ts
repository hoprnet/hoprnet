import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { HoprToken } from '../src/types'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment, maxFeePerGas, maxPriorityFeePerGas } = hre
  const environmentConfig = PROTOCOL_CONFIG.environments[environment]
  const mintedTokenReceiver = environmentConfig['minted_token_receiver_address']

  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const deployOptions = {
    log: true
  }

  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    deployOptions['waitConfirmations'] = 2
  }

  const tokenContract = await deployments.deploy('HoprToken', {
    from: deployer.address,
    maxFeePerGas,
    maxPriorityFeePerGas,
    ...deployOptions
  })

  if (network.tags.testing || network.tags.development) {
    const hoprToken = (await ethers.getContractFactory('HoprToken')).attach(tokenContract.address) as HoprToken
    const MINTER_ROLE = await hoprToken.MINTER_ROLE()
    const isDeployerMinter = await hoprToken.hasRole(MINTER_ROLE, deployer.address)

    if (!isDeployerMinter) {
      console.log('Granting MINTER role to', deployer.address)
      const grantTx = await hoprToken.grantRole(MINTER_ROLE, deployer.address)

      // don't wait when using local hardhat because its using auto-mine
      if (!environment.match('hardhat')) {
        await ethers.provider.waitForTransaction(grantTx.hash, 2)
      }

      if (mintedTokenReceiver) {
        console.log('Minting tokens to', mintedTokenReceiver)
        const mintTx = await hoprToken.mint(
          mintedTokenReceiver,
          ethers.utils.parseEther('130000000'),
          ethers.constants.HashZero,
          ethers.constants.HashZero
        )

        // don't wait when using local hardhat because its using auto-mine
        if (!environment.match('hardhat')) {
          await ethers.provider.waitForTransaction(mintTx.hash, 2)
        }
      }
    }
  }
}

// this smart contract should not be redeployed on a production or staging network
main.skip = async (env) => !!env.network.tags.production || !!env.network.tags.staging
main.dependencies = ['preDeploy']
main.tags = ['HoprToken']

export default main
