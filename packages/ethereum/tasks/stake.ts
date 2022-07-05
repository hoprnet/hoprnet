import { Signer, Wallet } from 'ethers'
import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { DEV_NFT_BOOST, DEV_NFT_TYPE } from '../utils/constants'

export type StakeOpts =
  | {
      type: 'devnft'
      privatekey: string // private key of the caller
    }
  | {
      type: 'xhopr'
      amount: string // target amount in wei
      privatekey: string // private key of the caller
    }

/**
 * Let caller to stake HOPR tokens in the current staking program
 */
async function main(
  opts: StakeOpts,
  { ethers, deployments, environment }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  // get envirionment
  if (environment == undefined) {
    console.error(`HOPR_ENVIRONMENT_ID is not set. Run with "HOPR_ENVIRONMENT_ID=<environment> ..."`)
    process.exit(1)
  }

  // get the provider and signer
  let signer: Signer
  if (!opts.privatekey) {
    signer = ethers.provider.getSigner()
  } else {
    signer = new Wallet(opts.privatekey, ethers.provider)
  }
  const signerAddress = await signer.getAddress()

  console.log('Signer Address', signerAddress)

  // get staking contract
  const stakingContract = await deployments.get('HoprStake')
  const hoprStake = (await ethers.getContractFactory('HoprStakeSeason3'))
    .connect(signer)
    .attach(stakingContract.address)

  if (opts.type === 'devnft') {
    // check if dev nft exists
    const nftContract = await deployments.get('HoprBoost')
    const hoprBoost = (await ethers.getContractFactory('HoprBoost')).connect(signer).attach(nftContract.address)
    // check if the signer has staked Dev NFT
    const hasStaked = await hoprStake.isNftTypeAndRankRedeemed4(DEV_NFT_TYPE, DEV_NFT_BOOST, signerAddress)
    // get Dev NFT index
    let devNFTIndex: number | null = null
    let loopCompleted = false
    let index = 0
    // loop through the array storage and record the length and dev nft index, if any
    while (!loopCompleted) {
      try {
        const createdNftTypes = await hoprBoost.typeAt(index + 1) // array of types are 1-based
        if (createdNftTypes === DEV_NFT_TYPE) {
          devNFTIndex = index
        }
      } catch (error) {
        // reaching the end of nft index array storage: panic code 0x32 (Array accessed at an out-of-bounds or negative index
        if (!(`${error}`.match(/0x32/g) || `${error}`.match(/cannot estimate gas/g))) {
          console.error(`Error in checking HoprBoost types. ${error}`)
        }
        loopCompleted = true
      }
      index++
    }

    if (!devNFTIndex) {
      console.error(`Cannot find Dev NFT index when staking.`)
      process.exit(1)
    } 

    if (hasStaked) {
      // Caller has staked Dev NFT, no need to repeat the process.
      console.log(`Address ${signerAddress} has staked Dev NFT. No need to stake more.`)
    }

    // check if the signer has Dev NFT
    const boostNFTBalance = await hoprBoost.balanceOf(signerAddress)
    let nftFound = false
    let nftFindingIndex = ethers.constants.Zero
    let ownedNFTTokenId = ethers.constants.Zero

    while (nftFindingIndex.lt(boostNFTBalance) && !nftFound) {
      ownedNFTTokenId = await hoprBoost.tokenOfOwnerByIndex(signerAddress, nftFindingIndex)
      const ownedNFTType = await hoprBoost.typeIndexOf(ownedNFTTokenId)
      if (ownedNFTType.eq(devNFTIndex)) {
        nftFound = true
      }
    }

    if (!nftFound) {
      // exit when the account does not own Dev NFT
      console.error(`Account ${signerAddress} does not have enough Dev NFT to proceed with staking`)
      process.exit(1)
    }

    // now the caller has Dev NFT and no Dev NFT has been staked. Now proceed with staking
    await (
      await hoprBoost['safeTransferFrom(address,address,uint256)'](
        signerAddress,
        stakingContract.address,
        ownedNFTTokenId
      )
    ).wait()
    console.log(`Address ${signerAddress} succeeded in staking Dev NFT.`)
  } else if (opts.type === 'xhopr' && opts.amount) {
    // must provide amount when token type is 'xhopr'
    const tokenContract = await deployments.get('xHoprMock')
    const hoprToken = (await ethers.getContractFactory('ERC677Mock')).connect(signer).attach(tokenContract.address)

    const balanceNativeToken = await ethers.provider.getBalance(signerAddress)
    let balanceHoprToken
    try {
      balanceHoprToken = await hoprToken.balanceOf(signerAddress)
    } catch (_) {
      balanceHoprToken = ethers.constants.Zero
    }
    console.log(`Account ${signerAddress} has ${balanceHoprToken} HOPR tokens`)
    console.log(`Account ${signerAddress} has ${balanceNativeToken} native tokens`)

    if (balanceNativeToken.lte(0)) {
      console.log(`Account ${signerAddress} does not have enough native tokens to proceed`)
      process.exit(1)
    }

    let stakedAmount
    try {
      stakedAmount = await hoprStake.stakedHoprTokens(signerAddress)
      console.log(`Account ${signerAddress} has staked ${stakedAmount}.`)
    } catch (_) {
      stakedAmount = ethers.constants.Zero
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
  } else {
    console.error('Missing argument when staking (Dev NFT/xHOPR) tokens.')
    process.exit(1)
  }
}

export default main
