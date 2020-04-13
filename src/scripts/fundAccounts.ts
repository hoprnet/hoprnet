import Web3 from 'web3'
import { BOOTSTRAP_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { stringToU8a } from '@hoprnet/hopr-utils'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import { HoprToken as IHoprToken } from '../tsc/web3/HoprToken'
import { privKeyToPubKey, pubKeyToAccountId } from '../utils'
import { AccountId } from '../types'
import { DEFAULT_URI, TOKEN_ADDRESSES, FUND_ACCOUNT_PRIVATE_KEY, DEMO_ACCOUNTS } from '../config'

const ACCOUNTS = [].concat(DEMO_ACCOUNTS, BOOTSTRAP_SEEDS)
const AMOUNT = Web3.utils.toWei('100', 'ether')

const privKeyToAddress = async (privKey: string) => {
  return privKeyToPubKey(stringToU8a(privKey))
    .then(pubKeyToAccountId)
    .then(address => new AccountId(address).toHex())
}

async function main() {
  const web3 = new Web3(DEFAULT_URI)
  const hoprToken: IHoprToken = new web3.eth.Contract(HoprTokenAbi as any, TOKEN_ADDRESSES.private)
  const owner = await privKeyToAddress(FUND_ACCOUNT_PRIVATE_KEY)

  for (const privKey of ACCOUNTS) {
    const address = await privKeyToAddress(privKey)

    await hoprToken.methods.mint(address, AMOUNT).send({
      from: owner,
      gas: 200e3
    })

    console.log(`funded ${address}`)
  }

  // TODO: check if this is needed
  process.exit()
}

main().catch(console.error)
