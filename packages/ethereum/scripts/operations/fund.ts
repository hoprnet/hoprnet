import Web3 from 'web3'
import HoprTokenAbi from '../../build/extracted/abis/HoprToken.json'
import { HOPR_TOKEN } from '../addresses'
import networks from '../../truffle-networks'

const AMOUNT = Web3.utils.toWei('1000000', 'ether')

export default async (amount?: number) => {
  const web3 = new Web3(`ws://${networks.development.host}:${networks.development.port}`)
  const hoprToken: any = new web3.eth.Contract(HoprTokenAbi as any, HOPR_TOKEN.private)

  const accounts = await web3.eth.getAccounts()
  const owner = accounts[0]

  if (amount && amount > accounts.length) {
    throw Error('Not enough demo secrets available.')
  }

  for (const account of accounts.slice(0, amount)) {
    await hoprToken.methods.mint(account, AMOUNT, '0x00', '0x00').send({
      from: owner,
      gas: 200e3,
    })

    console.log(`funded ${account}`)
  }

  // @ts-ignore
  web3.currentProvider.disconnect()
}
