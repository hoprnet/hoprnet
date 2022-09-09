import { Signer, Wallet } from 'ethers'
import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { NetworkRegistryNftRank, NR_NFT_TYPE, NR_NFT_TYPE_INDEX } from '../utils/constants'

export type StakeOpts =
  | {
      type: 'xhopr'
      amount: string // target amount in wei
      privatekey?: string // private key of the caller
    }
  | {
      type: 'nrnft'
      nftrank: NetworkRegistryNftRank
      privatekey?: string // private key of the caller
    }

async function stakeXhopr(hre: HardhatRuntimeEnvironment, signer, hoprStake, amount) {
  const { ethers, deployments } = hre
  // must provide amount when token type is 'xhopr'
  const tokenContract = await deployments.get('xHoprToken')
  const hoprToken = (await ethers.getContractFactory('ERC677Mock')).connect(signer).attach(tokenContract.address)

  const signerAddress = await signer.getAddress()
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

  if (ethers.BigNumber.from(stakedAmount).gte(ethers.BigNumber.from(amount))) {
    console.log(`Account ${signerAddress} has staked enough.`)
    return
  }

  const amountToStake = ethers.BigNumber.from(amount).sub(ethers.BigNumber.from(stakedAmount))
  console.log(`Account ${signerAddress} has to stake ${amount} HOPR tokens`)

  if (amountToStake.gt(ethers.BigNumber.from(balanceHoprToken))) {
    console.log(`Account ${signerAddress} does not have enough HOPR tokens to proceed with staking`)
    process.exit(1)
  }

  await (await hoprToken.transferAndCall(hoprStake.address, amountToStake, ethers.constants.HashZero)).wait()
  console.log(`Account ${signerAddress} staked ${amount} HOPR tokens successfully`)
}

/**
 * @notice nftTypeIndex is fixed as the must specify the nft rank.
 */
async function stakeNrNft(hre: HardhatRuntimeEnvironment, signer, hoprStake, nftrank: NetworkRegistryNftRank) {
  const { ethers, deployments } = hre
  // check if dev nft exists
  const nftContract = await deployments.get('HoprBoost')
  const hoprBoost = (await ethers.getContractFactory('HoprBoost')).connect(signer).attach(nftContract.address)
  // check if the signer has staked Network_registry NFT
  const signerAddress = await signer.getAddress()
  const hasStaked = await hoprStake.isNftTypeAndRankRedeemed2(NR_NFT_TYPE_INDEX, nftrank, signerAddress)
  const tokenUriSuffix = NR_NFT_TYPE + '/' + nftrank

  if (hasStaked) {
    // Caller has staked Network_registry NFT, no need to repeat the process.
    console.log(`Address ${signerAddress} has staked Network_registry NFT. No need to stake more.`)
    return
  }

  // check if the signer has desired Network_registry NFT
  const boostNFTBalance = await hoprBoost.balanceOf(signerAddress)
  let nftFound = false
  let nftFindingIndex = ethers.constants.Zero
  let ownedNFTTokenId = ethers.constants.Zero

  while (nftFindingIndex.lt(boostNFTBalance) && !nftFound) {
    ownedNFTTokenId = await hoprBoost.tokenOfOwnerByIndex(signerAddress, nftFindingIndex)
    const ownedNftUri = await hoprBoost.tokenURI(ownedNFTTokenId)
    if (new RegExp(tokenUriSuffix, 'g').test(ownedNftUri)) {
      console.log(`Found usable Network_registry NFT at index ${nftFindingIndex}`)
      nftFound = true
    }
    nftFindingIndex = nftFindingIndex.add(ethers.constants.One)
  }

  if (!nftFound) {
    // exit when the account does not own Network_registry NFT
    console.error(`Account ${signerAddress} does not have enough Network_registry NFT to proceed with staking`)
    process.exit(1)
  }

  // now the caller has Network_registry NFT and no Network_registry NFT has been staked. Now proceed with staking
  await (
    await hoprBoost['safeTransferFrom(address,address,uint256)'](signerAddress, hoprStake.address, ownedNFTTokenId)
  ).wait()
  console.log(`Address ${signerAddress} succeeded in staking Network_registry NFT.`)
}

/**
 * Let caller to stake HOPR tokens in the current staking program
 */
async function main(opts: StakeOpts, hre: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  const { ethers, deployments, environment } = hre
  // get envirionment
  if (environment == undefined) {
    console.error(`HOPR_ENVIRONMENT_ID is not set. Run with "HOPR_ENVIRONMENT_ID=<environment> ..."`)
    process.exit(1)
  }

  let provider
  if (environment == 'hardhat-localhost') {
    // we use a custom ethers provider here instead of the ethers object from the
    // hre which is managed by hardhat-ethers, because that one seems to
    // run its own in-memory hardhat instance, which is undesirable
    provider = new ethers.providers.JsonRpcProvider()
  } else {
    provider = ethers.provider
  }

  // get the provider and signer
  let signer: Signer
  if (!opts.privatekey) {
    signer = provider.getSigner()
  } else {
    signer = new Wallet(opts.privatekey, provider)
  }
  const signerAddress = await signer.getAddress()
  console.log('Signer Address', signerAddress)

  // get staking contract
  const stakingContract = await deployments.get('HoprStake')
  const hoprStake = (await ethers.getContractFactory('HoprStakeSeason3'))
    .connect(signer)
    .attach(stakingContract.address)

  if (opts.type === 'nrnft') {
    await stakeNrNft(hre, signer, hoprStake, opts.nftrank)
  } else if (opts.type === 'xhopr') {
    if (opts.amount) {
      await stakeXhopr(hre, signer, hoprStake, opts.amount)
    } else {
      console.error('Missing argument --amount when staking xHOPR tokens')
      process.exit(1)
    }
  } else {
    console.error(`Unsupported staking type ${JSON.stringify(opts)}`)
    process.exit(1)
  }
}

export default main
