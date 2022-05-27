import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'
import type { HoprBoost, HoprToken } from '../src/types'
import { utils } from 'ethers'
import { CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES, DEV_NFT_BOOST, MIN_STAKE } from '../utils/constants'
import type { HoprStakingProxyForNetworkRegistry } from '../src/types'

const DEV_NFT_TYPE = 'Dev'
const NUM_DEV_NFT = 3
const DUMMY_NFT_TYPE = 'Dummy'
const DUMMY_NFT_BOOST = 10
const MINTER_ROLE = utils.keccak256(utils.toUtf8Bytes('MINTER_ROLE'))
const DEFAULT_ADMIN_ROLE = '0x0000000000000000000000000000000000000000000000000000000000000000'
const NFT_BLOCKED = utils.keccak256(utils.toUtf8Bytes('NftBlocked(uint256)'))
const DEV_BANK_ADDRESS = '0x2402da10A6172ED018AEEa22CA60EDe1F766655C'

const main: DeployFunction = async function ({
  ethers,
  deployments,
  getNamedAccounts,
  network
}: HardhatRuntimeEnvironment) {
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
  let devNftIndex = null
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
      if (index === devNftIndex) {
        await hoprBoost.batchMint(new Array(NUM_DEV_NFT).fill(admin), DEV_NFT_TYPE, DEV_NFT_TYPE, DEV_NFT_BOOST, 0, {
          gasLimit: 4e6
        })
        console.log(`... minting ${NUM_DEV_NFT} ${DEV_NFT_TYPE} NFTs type of index ${index}`)
      } else {
        await hoprBoost.mint(admin, `${DUMMY_NFT_TYPE}_${index}`, DUMMY_NFT_TYPE, DUMMY_NFT_BOOST, 0, { gasLimit: 4e6 })
        console.log(`... minting 1 ${DUMMY_NFT_TYPE} NFTs type of index ${index}`)
      }
      index++
    }

    console.log(`Admin ${admin} has ${await hoprBoost.balanceOf(admin)} Boost NFTs`)
    // // renounce its MINTER_ROLE, if needed
    // await hoprBoost.renounceRole(MINTER_ROLE, deployer);
  }

  if (network.tags.staging) {
    // add special NFT types (dev NFTs) in network registry for staging envionment
    const registryProxy = (await ethers.getContractFactory('HoprStakingProxyForNetworkRegistry')).attach(
      registryProxyDeployment.address
    ) as HoprStakingProxyForNetworkRegistry
    await registryProxy.ownerBatchAddNftTypeAndRank([devNftIndex], [DEV_NFT_BOOST])

    try {
      // mint minimum stake to addresses that will stake and are binded to nodes in NR
      const tokenContract = await deployments.get('HoprToken')
      const hoprToken = (await ethers.getContractFactory('HoprToken')).attach(tokenContract.address) as HoprToken
      await hoprToken.mint(
        CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[0],
        MIN_STAKE,
        ethers.constants.HashZero,
        ethers.constants.HashZero
      )
      await hoprToken.mint(
        CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[2],
        MIN_STAKE,
        ethers.constants.HashZero,
        ethers.constants.HashZero
      )
      console.log(`... minting ${MIN_STAKE} txHOPR to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[0] and [2]`)
    } catch (error) {
      console.error(
        `Cannot mint txHOPR to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[0] and CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[2] due to ${error}`
      )
    }

    try {
      await hoprBoost.batchMint(
        [CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[1], CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[3]],
        DEV_NFT_TYPE,
        DEV_NFT_TYPE,
        DEV_NFT_BOOST,
        0,
        {
          gasLimit: 4e6
        }
      )
      console.log(`... minting ${DEV_NFT_TYPE} NFTs to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[1] and [3]`)
    } catch (error) {
      console.error(
        `Cannot mint ${DEV_NFT_TYPE} NFTs to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[1] and CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[3] due to ${error}`
      )
    }
  }

  const isDeployerAdmin = await hoprBoost.hasRole(DEFAULT_ADMIN_ROLE, deployer)
  if (isDeployerAdmin) {
    // Assign the Dev Bank as a minter role for HOPR Boost
    await hoprBoost.grantRole(MINTER_ROLE, DEV_BANK_ADDRESS)
    if (deployer !== admin) {
      // make admin MINTER
      await hoprBoost.grantRole(MINTER_ROLE, admin)
      // transfer DEFAULT_ADMIN_ROLE from deployer to admin
      await hoprBoost.grantRole(DEFAULT_ADMIN_ROLE, admin)
      console.log('DEFAULT_ADMIN_ROLE is transferred.')
      await hoprBoost.renounceRole(DEFAULT_ADMIN_ROLE, deployer)
      console.log('DEFAULT_ADMIN_ROLE is transferred.')
    }
  }
}

main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production

export default main
