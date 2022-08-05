import { type Signer, type Contract, Wallet } from 'ethers'
import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { DEV_NFT_BOOST, DEV_NFT_TYPE } from '../utils/constants'

export type RequestTestTokensOpts = {
  type: 'devnft' | 'xhopr'
  amount?: string // target amount in wei
  recipient: string // address of the recipient
  privatekey: string // private key of the Boost NFT owner
}

async function requestXhopr(hre: HardhatRuntimeEnvironment, signer: Signer, amount: string, recipientAddress: string) {
  const { ethers, deployments } = hre
  // must provide amount when token type is 'xhopr'
  const tokenContract = await deployments.get('xHoprMock')
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

async function requestDevNft(hre: HardhatRuntimeEnvironment, signer: Signer, hoprStake: Contract, recipientAddress: string) {
  const { ethers, deployments } = hre
  const signerAddress = await signer.getAddress()

  // check if dev nft exists
  const nftContract = await deployments.get('HoprBoost')
  const hoprBoost = (await ethers.getContractFactory('HoprBoost')).connect(signer).attach(nftContract.address)

  // check if the recipient has staked Dev NFT
  const hasStaked = await hoprStake.isNftTypeAndRankRedeemed4(DEV_NFT_TYPE, DEV_NFT_BOOST, recipientAddress)

  if (hasStaked) {
    // Recipient has staked Dev NFT, not going to send NFT again.
    console.log(`Address ${recipientAddress} has staked Dev NFT. No need to request more.`)
    return
  }

  // get Dev NFT index
  let devNFTIndex: number | null = null
  let index = 1
  // loop through the array storage and record the length and dev nft index, if any
  while (true) {
    try {
      console.error(`Check DevNFT type at index = ${index}`)
      const createdNftTypes = await hoprBoost.typeAt(index) // array of types are 1-based
      if (createdNftTypes === DEV_NFT_TYPE) {
        console.error(`Found usable DevNFT type at index = ${index}`)
        devNFTIndex = index
        break
      }
    } catch (error) {
      // reaching the end of nft index array storage: panic code 0x32 (Array accessed at an out-of-bounds or negative index
      if (!(`${error}`.match(/0x32/g) || `${error}`.match(/cannot estimate gas/g))) {
        console.error(`Error in checking HoprBoost types. ${error}`)
      }
      console.error(`Completed DevNFT check loop without result`)
      break
    }
    index++
  }

  if (!devNFTIndex) {
    console.error(`Cannot find Dev NFT index before sending.`)
    process.exit(1)
  }

  // check if the recipient has Dev NFT
  const recipientBoostNFTBalance = await hoprBoost.balanceOf(recipientAddress)
  let recipientNftFound = false
  let recipientNftFindingIndex = ethers.constants.Zero
  let recipientOwnedNFTTokenId = ethers.constants.Zero

  while (recipientNftFindingIndex.lt(recipientBoostNFTBalance) && !recipientNftFound) {
    recipientOwnedNFTTokenId = await hoprBoost.tokenOfOwnerByIndex(signerAddress, recipientNftFindingIndex)
    const ownedNFTType = await hoprBoost.typeIndexOf(recipientOwnedNFTTokenId)
    if (ownedNFTType.eq(devNFTIndex)) {
      console.log(`Found usable DevNFT at index ${recipientNftFindingIndex}`)
      recipientNftFound = true
    }
    recipientNftFindingIndex = recipientNftFindingIndex.add(ethers.constants.One)
  }

  if (recipientNftFound) {
    // exit when the account already owns Dev NFT
    console.log(`Account ${recipientAddress} already has Dev NFT. Stop sending`)
    return
  }

  // check if the signer has Dev NFT
  const signerBoostNFTBalance = await hoprBoost.balanceOf(signerAddress)
  let signerNftFound = false
  let signerNftFindingIndex = ethers.constants.Zero
  let signerOwnedNFTTokenId = ethers.constants.Zero

  while (signerNftFindingIndex.lt(signerBoostNFTBalance) && !signerNftFound) {
    signerOwnedNFTTokenId = await hoprBoost.tokenOfOwnerByIndex(signerAddress, signerNftFindingIndex)
    const ownedNFTType = await hoprBoost.typeIndexOf(signerOwnedNFTTokenId)
    if (ownedNFTType.eq(devNFTIndex)) {
      console.log(`Found usable DevNFT at index ${signerNftFindingIndex}`)
      signerNftFound = true
    }
    signerNftFindingIndex = signerNftFindingIndex.add(ethers.constants.One)
  }

  if (!signerNftFound) {
    // exit when the account does not own Dev NFT
    console.error(`Account ${signerAddress} does not have enough Dev NFT to proceed with sending`)
    process.exit(1)
  }

  // now the caller has Dev NFT and no Dev NFT has been staked. Now proceed with staking
  await (
    await hoprBoost['safeTransferFrom(address,address,uint256)'](signerAddress, recipientAddress, signerOwnedNFTTokenId)
  ).wait()
  console.log(`Address ${recipientAddress} succeeded in receiving Dev NFT from ${signerAddress}.`)
}

/**
 * As a prerequisite for staking, the staker must request Dev NFT. Staker is the recipient. privatekey should
 * be the private key of an account that owns some Dev NFTs
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

  if (opts.type === 'devnft') {
    await requestDevNft(hre, signer, hoprStake, recipientAddress)
  } else if (opts.type === 'xhopr') {
    if (opts.amount) {
      await requestXhopr(hre, signer, opts.amount, recipientAddress)
    } else {
      console.error('Missing argument --amount when requesting xHOPR tokens')
      process.exit(1)
    }
  } else {
    console.error(`Unsupported requesting type ${opts.type}`)
    process.exit(1)
  }
}

export default main
