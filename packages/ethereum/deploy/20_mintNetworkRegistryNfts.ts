import { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { HoprBoost, HoprStakingProxyForNetworkRegistry } from '../src/types'
import { type ContractTransaction, utils } from 'ethers'
import {
  CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES,
  NR_NFT_BOOST,
  NR_NFT_MAX_REGISTRATION_COM,
  NR_NFT_MAX_REGISTRATION_TECH,
  NR_NFT_RANK_COM,
  NR_NFT_RANK_TECH,
  NR_NFT_TYPE,
  NR_NFT_TYPE_INDEX
} from '../utils/constants'

const NUM_NR_NFT = 3
const DUMMY_NFT_TYPE = 'Dummy'
const DUMMY_NFT_BOOST = 10
const MINTER_ROLE = utils.keccak256(utils.toUtf8Bytes('MINTER_ROLE'))
const DEFAULT_ADMIN_ROLE = '0x0000000000000000000000000000000000000000000000000000000000000000'
const DEV_BANK_ADDRESS = '0x2402da10A6172ED018AEEa22CA60EDe1F766655C'

const getToCreateDummyNftIndexes = async (
  hoprBoost: HoprBoost,
  shouldHaveIndexesBefore: number
): Promise<Array<number>> => {
  // get nft types created in the HoprBoost contract
  let mintedIndex = 0
  // loop through the array storage and record the length and dev nft index, if any
  while (mintedIndex < shouldHaveIndexesBefore) {
    try {
      const createdNftTypes = await hoprBoost.typeAt(mintedIndex + 1, { gasLimit: 400e3 }) // array of types are 1-based
      console.log(`createdNftTypes ${createdNftTypes}`)
    } catch (error) {
      // reaching the end of nft index array storage: panic code 0x32 (Array accessed at an out-of-bounds or negative index
      if (`${error}`.match(/0x32/g) || `${error}`.match(/cannot estimate gas/g)) {
        break
      } else {
        console.log(`Error in checking HoprBoost types. ${error}`)
      }
    }
    mintedIndex++
  }

  const dummyNftIndexsToMint =
    mintedIndex > shouldHaveIndexesBefore
      ? []
      : Array.from({ length: shouldHaveIndexesBefore - mintedIndex + 1 }, (_, i) => i + mintedIndex)

  console.log(
    `To have HoprBoost NFT of ${NR_NFT_TYPE} type at index ${shouldHaveIndexesBefore}, ${dummyNftIndexsToMint.length} type(s) of dummy NFTs of indexes ${dummyNftIndexsToMint} should be minted.`
  )

  return dummyNftIndexsToMint
}

const awaitTxConfirmation = async (
  tx: Promise<ContractTransaction>,
  hreEnvirionment: string,
  hreEthers: HardhatRuntimeEnvironment['ethers']
) => {
  const mintTx = await tx
  // don't wait when using local hardhat because its using auto-mine
  if (!hreEnvirionment.match('hardhat')) {
    await hreEthers.provider.waitForTransaction(mintTx.hash, 2)
  }
}

/**
 *
 * @notice This script should only be run in staging/hardhat-localhost envirionment
 * @param hre
 */

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, environment, network } = hre
  const { deployer, admin } = await getNamedAccounts()

  // check boost types being created
  const boostDeployment = await deployments.get('HoprBoost')
  const registryProxyDeployment = await deployments.get('HoprNetworkRegistryProxy')
  const hoprBoost = (await ethers.getContractFactory('HoprBoost')).attach(boostDeployment.address) as HoprBoost

  // check type index of HoprBoost NFTs
  const dummyNftTypesToBeMinted = await getToCreateDummyNftIndexes(hoprBoost, NR_NFT_TYPE_INDEX)

  const isDeployerMinter = await hoprBoost.hasRole(MINTER_ROLE, deployer)
  if (isDeployerMinter) {
    // current deployer is minter, mint
    console.log('Deployer is a minter. Mint necessary HoprBoost NFTs')

    if (dummyNftTypesToBeMinted.length > 0) {
      // need to mint dummy NFTs
      for (const dummyNftIndex of dummyNftTypesToBeMinted) {
        console.log(`... minting 1 ${DUMMY_NFT_TYPE} NFTs type of rank ${DUMMY_NFT_TYPE}`)
        await awaitTxConfirmation(
          hoprBoost.mint(admin, `${DUMMY_NFT_TYPE}_${dummyNftIndex}`, DUMMY_NFT_TYPE, DUMMY_NFT_BOOST, 0, {
            gasLimit: 4e6
          }),
          environment,
          hre.ethers
        )
      }
    }

    // mint NR NFTs
    for (const networkRegistryNftRank of [NR_NFT_RANK_TECH, NR_NFT_RANK_COM]) {
      console.log(
        `... minting ${NUM_NR_NFT} ${NR_NFT_TYPE} NFTs type of index ${NR_NFT_TYPE_INDEX} to ${admin}\n...minting 1 ${NR_NFT_TYPE} NFTs to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES\n...minting 10 ${NR_NFT_TYPE} NFTs to dev bank ${DEV_BANK_ADDRESS}`
      )
      await awaitTxConfirmation(
        hoprBoost.batchMint(
          [
            ...new Array(NUM_NR_NFT).fill(admin),
            ...CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES,
            ...Array(10).fill(DEV_BANK_ADDRESS)
          ],
          NR_NFT_TYPE,
          networkRegistryNftRank,
          NR_NFT_BOOST,
          0,
          {
            gasLimit: 4e6
          }
        ),
        environment,
        ethers
      )
    }

    console.log(`Admin ${admin} has ${await hoprBoost.balanceOf(admin)} Boost NFTs`)
  } else {
    console.log(`Deployer is not minter. Skip minting NFTs, although ${dummyNftTypesToBeMinted} need to be minted.`)
  }

  // Add special NFTs in staging environment (for staging environment)
  if (network.tags.staging) {
    // add special NFT types (dev NFTs) in network registry for staging
    const registryProxy = (await ethers.getContractFactory('HoprStakingProxyForNetworkRegistry')).attach(
      registryProxyDeployment.address
    ) as HoprStakingProxyForNetworkRegistry

    await awaitTxConfirmation(
      registryProxy.ownerBatchAddSpecialNftTypeAndRank(
        [NR_NFT_TYPE_INDEX, NR_NFT_TYPE_INDEX],
        [NR_NFT_RANK_TECH, NR_NFT_RANK_COM],
        [NR_NFT_MAX_REGISTRATION_TECH, NR_NFT_MAX_REGISTRATION_COM],
        {
          gasLimit: 4e6
        }
      ),
      environment,
      ethers
    )

    // // currently we don't use funds, only NFTs
    // await awaitTxConfirmation(
    //   registryProxy.ownerBatchAddNftTypeAndRank(
    //     [NR_NFT_TYPE_INDEX, NR_NFT_TYPE_INDEX],
    //     [NR_NFT_RANK_TECH, NR_NFT_RANK_COM]
    //   ),
    //   environment,
    //   ethers
    // )
    // try {
    //   // mint minimum stake to addresses that will stake and are binded to nodes in NR
    //   const tokenContract = await deployments.get('xHoprToken')
    //   const hoprToken = (await ethers.getContractFactory('ERC677Mock')).attach(tokenContract.address) as ERC677Mock

    //   await awaitTxConfirmation(
    //     hoprToken.batchMintInternal([CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[0]], MIN_STAKE),
    //     environment,
    //     ethers
    //   )
    //   await awaitTxConfirmation(
    //     hoprToken.batchMintInternal([CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[2]], MIN_STAKE),
    //     environment,
    //     ethers
    //   )
    //   console.log(`... minting ${MIN_STAKE} txHOPR to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[0] and [2]`)
    // } catch (error) {
    //   console.error(
    //     `Cannot mint txHOPR to CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[0] and CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[2] due to ${error}`
    //   )
    // }
  }

  const isDeployerAdmin = await hoprBoost.hasRole(DEFAULT_ADMIN_ROLE, deployer)
  if (isDeployerAdmin && deployer !== admin) {
    // make admin MINTER
    await awaitTxConfirmation(hoprBoost.grantRole(MINTER_ROLE, admin), environment, ethers)
    // transfer DEFAULT_ADMIN_ROLE from deployer to admin
    await awaitTxConfirmation(hoprBoost.grantRole(DEFAULT_ADMIN_ROLE, admin), environment, ethers)
    console.log('DEFAULT_ADMIN_ROLE is transferred.')

    await awaitTxConfirmation(hoprBoost.renounceRole(DEFAULT_ADMIN_ROLE, deployer), environment, ethers)
    console.log('DEFAULT_ADMIN_ROLE is transferred.')
  }
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistry', 'HoprBoost', 'HoprStake']
main.tags = ['MintNetworkRegistryNfts']

export default main
