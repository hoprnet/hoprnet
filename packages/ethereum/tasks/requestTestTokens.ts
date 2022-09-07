import { type Signer, type Contract, Wallet } from 'ethers'
import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { NetworkRegistryNftRank, NR_NFT_BOOST, NR_NFT_TYPE, NR_NFT_TYPE_INDEX } from '../utils/constants'

export type RequestTestTokensOpts =
  | {
      type: 'xhopr'
      amount: string // target amount in wei
      recipient: string // address of the recipient
      privatekey?: string // private key of the Boost NFT owner
    }
  | {
      type: 'nrnft'
      nftRank: NetworkRegistryNftRank
      recipient: string // address of the recipient
      privatekey?: string // private key of the Boost NFT owner
    }

async function requestXhopr(hre: HardhatRuntimeEnvironment, signer: Signer, amount: string, recipientAddress: string) {
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
  console.log(`DevBank account ${signerAddress} has ${balanceHoprToken} HOPR tokens`)
  console.log(`DevBank account ${signerAddress} has ${balanceNativeToken} native tokens`)

  if (balanceNativeToken.lte(0)) {
    console.log(`DevBank account ${signerAddress} does not have enough native tokens to proceed`)
    process.exit(1)
  }

  if (ethers.BigNumber.from(amount).gt(ethers.BigNumber.from(balanceHoprToken))) {
    console.log(`DevBank account ${signerAddress} does not have enough HOPR tokens to fulfill the request`)
    process.exit(1)
  }

  try {
    await (await hoprToken.transferAndCall(recipientAddress, amount, ethers.constants.HashZero)).wait()
    console.log(`DevBank account ${signerAddress} transferred ${amount} HOPR tokens successfully`)
  } catch (error) {
    console.error(`Requesting HOPR tokens failed due to ${error}`)
  }
}

async function requestNrNft(
  hre: HardhatRuntimeEnvironment,
  signer: Signer,
  hoprStake: Contract,
  nftRank: NetworkRegistryNftRank,
  recipientAddress: string
) {
  const { ethers, deployments } = hre
  const signerAddress = await signer.getAddress()

  // check if dev nft exists
  const nftContract = await deployments.get('HoprBoost')
  const hoprBoost = (await ethers.getContractFactory('HoprBoost')).connect(signer).attach(nftContract.address)

  // check if the recipient has staked Network_registry NFT
  const hasStaked = await hoprStake.isNftTypeAndRankRedeemed2(NR_NFT_TYPE_INDEX, NR_NFT_BOOST, recipientAddress)
  const tokenUriSuffix = NR_NFT_TYPE + '/' + nftRank

  if (hasStaked) {
    // Recipient has staked Network_registry NFT, not going to send NFT again.
    console.log(`Address ${recipientAddress} has staked Network_registry NFT. No need to request more.`)
    return
  }

  // check if the recipient has Network_registry NFT
  const recipientBoostNFTBalance = await hoprBoost.balanceOf(recipientAddress)
  let recipientNftFound = false
  let recipientNftFindingIndex = ethers.constants.Zero
  let recipientOwnedNFTTokenId = ethers.constants.Zero

  while (recipientNftFindingIndex.lt(recipientBoostNFTBalance) && !recipientNftFound) {
    recipientOwnedNFTTokenId = await hoprBoost.tokenOfOwnerByIndex(signerAddress, recipientNftFindingIndex)
    const recipientOwnedNftUri = await hoprBoost.tokenURI(recipientOwnedNFTTokenId)
    if (new RegExp(tokenUriSuffix, 'g').test(recipientOwnedNftUri)) {
      console.log(`Found usable Network_registry NFT at index ${recipientNftFindingIndex}`)
      recipientNftFound = true
    }
    recipientNftFindingIndex = recipientNftFindingIndex.add(ethers.constants.One)
  }

  if (recipientNftFound) {
    // exit when the account already owns Network_registry NFT
    console.log(`Account ${recipientAddress} already has Network_registry NFT. Stop sending`)
    return
  }

  // check if the signer has Network_registry NFT
  const signerBoostNFTBalance = await hoprBoost.balanceOf(signerAddress)
  let signerNftFound = false
  let signerNftFindingIndex = ethers.constants.Zero
  let signerOwnedNFTTokenId = ethers.constants.Zero

  while (signerNftFindingIndex.lt(signerBoostNFTBalance) && !signerNftFound) {
    signerOwnedNFTTokenId = await hoprBoost.tokenOfOwnerByIndex(signerAddress, signerNftFindingIndex)
    const signerOwnedNftUri = await hoprBoost.tokenURI(signerOwnedNFTTokenId)
    if (new RegExp(tokenUriSuffix, 'g').test(signerOwnedNftUri)) {
      console.log(`Found usable Network_registry NFT at index ${signerNftFindingIndex}`)
      signerNftFound = true
    }
    signerNftFindingIndex = signerNftFindingIndex.add(ethers.constants.One)
  }

  if (!signerNftFound) {
    // exit when the account does not own Network_registry NFT
    console.error(`Account ${signerAddress} does not have enough Network_registry NFT to proceed with sending`)
    process.exit(1)
  }

  // now the caller has Network_registry NFT and no Network_registry NFT has been staked. Now proceed with staking
  await (
    await hoprBoost['safeTransferFrom(address,address,uint256)'](signerAddress, recipientAddress, signerOwnedNFTTokenId)
  ).wait()
  console.log(`Address ${recipientAddress} succeeded in receiving Network_registry NFT from ${signerAddress}.`)
}

/**
 * As a prerequisite for staking, the staker must request Network_registry NFT. Staker is the recipient. privatekey should
 * be the private key of an account that owns some Network_registry NFTs
 */
async function main(opts: RequestTestTokensOpts, hre: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
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

  const recipientAddress = opts.recipient
  console.log('Recipient Address', recipientAddress)

  // get staking contract
  const stakingContract = await deployments.get('HoprStake')
  const hoprStake = (await ethers.getContractFactory('HoprStakeSeason3'))
    .connect(signer)
    .attach(stakingContract.address)

  if (opts.type === 'nrnft') {
    await requestNrNft(hre, signer, hoprStake, opts.nftRank, recipientAddress)
  } else if (opts.type === 'xhopr') {
    if (opts.amount) {
      await requestXhopr(hre, signer, opts.amount, recipientAddress)
    } else {
      console.error('Missing argument --amount when requesting xHOPR tokens')
      process.exit(1)
    }
  } else {
    console.error(`Unsupported requesting type ${JSON.stringify(opts)}`)
    process.exit(1)
  }
}

export default main
