import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'
import type { HoprBoost, ERC677Mock } from '../src/types'
import { utils } from 'ethers'
import { CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES, DEV_NFT_BOOST, DEV_NFT_TYPE, MIN_STAKE } from '../utils/constants'
import type { HoprStakingProxyForNetworkRegistry } from '../src/types'

const NUM_DEV_NFT = 3
const DUMMY_NFT_TYPE = 'Dummy'
const DUMMY_NFT_BOOST = 10
const MINTER_ROLE = utils.keccak256(utils.toUtf8Bytes('MINTER_ROLE'))
const DEFAULT_ADMIN_ROLE = '0x0000000000000000000000000000000000000000000000000000000000000000'
const NFT_BLOCKED = utils.keccak256(utils.toUtf8Bytes('NftBlocked(uint256)'))
const DEV_BANK_ADDRESS = '0x2402da10A6172ED018AEEa22CA60EDe1F766655C'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, environment } = hre
  const { deployer, admin } = await getNamedAccounts()

  // check boost types being created
  const stakeDeployment = await deployments.get('HoprStake')
  const boostDeployment = await deployments.get('HoprBoost')
  const registryProxyDeployment = await deployments.get('HoprNetworkRegistryProxy')
  const hoprBoost = (await ethers.getContractFactory('HoprBoost')).attach(boostDeployment.address) as HoprBoost

  // find the blocked NFT types and check the created boost types
  const blockedNftTypes = stakeDeployment.receipt.logs
    .filter((log) => log.topics[0] === NFT_BLOCKED)
    .map((log) => parseInt(log.topics[1]))

  // get max blocked nft type index
  const blockNftTypeMax = blockedNftTypes.reduce((a, b) => Math.max(a, b))
  // get nft types created in the HoprBoost contract
  let devNftIndex: number | null = null
  let loopCompleted = false
  let index = 0
  // loop through the array storage and record the length and dev nft index, if any
  while (!loopCompleted) {
    try {
      const createdNftTypes = await hoprBoost.typeAt(index + 1, { gasLimit: 400e3 }) // array of types are 1-based
      console.log(`createdNftTypes ${createdNftTypes}`)
      if (createdNftTypes === DEV_NFT_TYPE) {
        devNftIndex = index
      }
    } catch (error) {
      // reaching the end of nft index array storage: panic code 0x32 (Array accessed at an out-of-bounds or negative index
      if (`${error}`.match(/0x32/g) || `${error}`.match(/cannot estimate gas/g)) {
        loopCompleted = true
      } else {
        console.log(`Error in checking HoprBoost types. ${error}`)
      }
    }
    index++
  }
  // assign the dev nft if dev nft does not exist.
  if (!devNftIndex) {
    devNftIndex = Math.max(blockNftTypeMax, index) + 1
  }

  console.log(
    `HoprBoost NFT now has ${
      index - 1
    } types and should have at least ${blockNftTypeMax} types, where ${blockedNftTypes} are blocked. Dev NFT should be at ${devNftIndex}`
  )

  const isDeployerMinter = await hoprBoost.hasRole(MINTER_ROLE, deployer)
  if (isDeployerMinter) {
    // current deployer is minter
    console.log('Deployer is a minter. Mint necessary HoprBoost NFTs')

    // mint all the dummy NFTs (especially those are blocked in the constructor). Mint some dev NFTs
    while (index <= blockNftTypeMax || index <= devNftIndex) {
      console.log(`Minting type of index ${index}`)
      let mintTx
      if (index === devNftIndex) {
        mintTx = await hoprBoost.batchMint(
          new Array(NUM_DEV_NFT).fill(admin),
          DEV_NFT_TYPE,
          DEV_NFT_TYPE,
          DEV_NFT_BOOST,
          0,
          {
            gasLimit: 4e6
          }
        )
        console.log(`... minting ${NUM_DEV_NFT} ${DEV_NFT_TYPE} NFTs type of index ${index}`)
      } else {
        mintTx = await hoprBoost.mint(admin, `${DUMMY_NFT_TYPE}_${index}`, DUMMY_NFT_TYPE, DUMMY_NFT_BOOST, 0, {
          gasLimit: 4e6
        })
        console.log(`... minting 1 ${DUMMY_NFT_TYPE} NFTs type of index ${index}`)
      }

      // don't wait when using local hardhat because its using auto-mine
      if (!environment.match('hardhat')) {
        await ethers.provider.waitForTransaction(mintTx.hash, 2)
      }

      index++
    }

    console.log(`Admin ${admin} has ${await hoprBoost.balanceOf(admin)} Boost NFTs`)
    // // renounce its MINTER_ROLE, if needed
    // await hoprBoost.renounceRole(MINTER_ROLE, deployer);
  }

  if (!environment.match('hardhat')) {
    // add special NFT types (dev NFTs) in network registry for staging envionment
    const registryProxy = (await ethers.getContractFactory('HoprStakingProxyForNetworkRegistry')).attach(
      registryProxyDeployment.address
    ) as HoprStakingProxyForNetworkRegistry
    const ownerAddDevNftTypeTx = await registryProxy.ownerBatchAddNftTypeAndRank([devNftIndex], [DEV_NFT_BOOST])
    const ownerAddSpecialNftTypeTx = await registryProxy.ownerBatchAddSpecialNftTypeAndRank(
      [devNftIndex],
      [DEV_NFT_BOOST]
    )

    // don't wait when using local hardhat because its using auto-mine
    if (!environment.match('hardhat')) {
      await ethers.provider.waitForTransaction(ownerAddDevNftTypeTx.hash, 2)
      await ethers.provider.waitForTransaction(ownerAddSpecialNftTypeTx.hash, 2)
    }

    try {
      // mint minimum stake to addresses that will stake and are binded to nodes in NR
      const tokenContract = await deployments.get('xHoprMock')
      const hoprToken = (await ethers.getContractFactory('ERC677Mock')).attach(tokenContract.address) as ERC677Mock

      const mintTx1 = await hoprToken.batchMintInternal([CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[0]], MIN_STAKE)
      // don't wait when using local hardhat because its using auto-mine
      if (!environment.match('hardhat')) {
        await ethers.provider.waitForTransaction(mintTx1.hash, 2)
      }

      const mintTx2 = await hoprToken.batchMintInternal([CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[2]], MIN_STAKE)
      // don't wait when using local hardhat because its using auto-mine
      if (!environment.match('hardhat')) {
        await ethers.provider.waitForTransaction(mintTx2.hash, 2)
      }

      console.log(`... minting ${MIN_STAKE} txHOPR to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[0] and [2]`)
    } catch (error) {
      console.error(
        `Cannot mint txHOPR to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[0] and CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[2] due to ${error}`
      )
    }

    try {
      const mintNftTx = await hoprBoost.batchMint(
        [
          CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[1],
          CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[3],
          ...Array(10).fill(DEV_BANK_ADDRESS)
        ],
        DEV_NFT_TYPE,
        DEV_NFT_TYPE,
        DEV_NFT_BOOST,
        0,
        {
          gasLimit: 4e6
        }
      )
      // don't wait when using local hardhat because its using auto-mine
      if (!environment.match('hardhat')) {
        await ethers.provider.waitForTransaction(mintNftTx.hash, 2)
      }

      console.log(
        `... minting ${DEV_NFT_TYPE} NFTs to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[1], [3] and 10 for dev bank`
      )
    } catch (error) {
      console.error(
        `Cannot mint ${DEV_NFT_TYPE} NFTs to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[1] and CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[3] due to ${error}`
      )
    }
  }

  const isDeployerAdmin = await hoprBoost.hasRole(DEFAULT_ADMIN_ROLE, deployer)
  if (isDeployerAdmin && deployer !== admin) {
    // make admin MINTER
    const grantMinterTx = await hoprBoost.grantRole(MINTER_ROLE, admin)
    // don't wait when using local hardhat because its using auto-mine
    if (!environment.match('hardhat')) {
      await ethers.provider.waitForTransaction(grantMinterTx.hash, 2)
    }

    // transfer DEFAULT_ADMIN_ROLE from deployer to admin
    const grantAdminTx = await hoprBoost.grantRole(DEFAULT_ADMIN_ROLE, admin)
    // don't wait when using local hardhat because its using auto-mine
    if (!environment.match('hardhat')) {
      await ethers.provider.waitForTransaction(grantAdminTx.hash, 2)
    }
    console.log('DEFAULT_ADMIN_ROLE is transferred.')

    const renounceAdminTx = await hoprBoost.renounceRole(DEFAULT_ADMIN_ROLE, deployer)
    // don't wait when using local hardhat because its using auto-mine
    if (!environment.match('hardhat')) {
      await ethers.provider.waitForTransaction(renounceAdminTx.hash, 2)
    }
    console.log('DEFAULT_ADMIN_ROLE is transferred.')
  }
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistry', 'HoprBoost', 'HoprStake']
main.tags = ['MintDevNftTransferOwnership']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production

export default main
