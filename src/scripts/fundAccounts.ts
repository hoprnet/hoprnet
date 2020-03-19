import Web3 from 'web3'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import { HoprToken as IHoprToken } from '../tsc/web3/HoprToken'
import { stringToU8a } from '../core/u8a'
import { privKeyToPubKey, pubKeyToAccountId } from '../utils'
import { AccountId } from '../types'
import { DEFAULT_URI, DEFAULT_HOPR_TOKEN_ADDRESS, FUND_ACCOUNT_PRIVATE_KEY, DEMO_ACCOUNTS } from '../config'

const AMOUNT = Web3.utils.toWei('100', 'ether')

const privKeyToAddress = async (privKey: string) => {
  return privKeyToPubKey(stringToU8a(privKey))
    .then(pubKeyToAccountId)
    .then(address => new AccountId(address).toHex())
}

async function main() {
  const web3 = new Web3(DEFAULT_URI)
  const hoprToken: IHoprToken = new web3.eth.Contract(HoprTokenAbi as any, DEFAULT_HOPR_TOKEN_ADDRESS)
  const owner = await privKeyToAddress(FUND_ACCOUNT_PRIVATE_KEY)

  for (const privKey of DEMO_ACCOUNTS) {
    const address = await privKeyToAddress(privKey)

    await hoprToken.methods.mint(address, AMOUNT).send({
      from: owner
    })
    console.log(`funded ${address}`)
  }
}

main().catch(console.error)
