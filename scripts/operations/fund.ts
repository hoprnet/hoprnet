import Web3 from 'web3'
import HoprTokenAbi from '../../build/extracted/abis/HoprToken.json'

const AMOUNT = Web3.utils.toWei('1000000', 'ether')

export default async () => {
  const web3 = new Web3(`ws://127.0.0.1:9545`)
  const hoprToken: any = new web3.eth.Contract(HoprTokenAbi as any, '0x302be990306f95a21905d411450e2466DC5DD927')

  const accounts = await web3.eth.getAccounts()
  const owner = accounts[0]

  for (const account of accounts) {
    await hoprToken.methods.mint(account, AMOUNT).send({
      from: owner,
      gas: 200e3,
    })

    console.log(`funded ${account}`)
  }

  // @ts-ignore
  web3.currentProvider.disconnect()
}
