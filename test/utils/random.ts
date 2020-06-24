import BigNumber from 'bignumber.js'
import BN from 'bn.js'
import Web3 from 'web3'

const web3 = new Web3()

export const MAX_UINT256 = new BigNumber(2).pow(256).minus(1)

export const keccak256 = (...args: { type: string; value: string | number }[]): string => {
  return Web3.utils.soliditySha3(...((args as unknown) as any))
}

export const signMessage = (web3: Web3, message: string, signerPrivKey: string) => {
  return web3.eth.accounts.sign(message, signerPrivKey)
}

export const recoverSigner = (web3: Web3, message: string, signature: string) => {
  return web3.eth.accounts.recover(message, signature, false)
}

export const createChallage = (a: string, b: string): string => {
  return keccak256({
    type: 'bytes',
    value: encode([
      {
        type: 'bytes32',
        value: a,
      },
      {
        type: 'bytes32',
        value: b,
      },
    ]),
  })
}

export const isPartyA = (accountA: string, accountB: string) => {
  return new BN(accountA.slice(2), 16).lt(new BN(accountB.slice(2), 16))
}

export const getParties = (accountA: string, accountB: string) => {
  if (isPartyA(accountA, accountB)) {
    return {
      partyA: accountA,
      partyB: accountB,
    }
  }

  return {
    partyA: accountB,
    partyB: accountA,
  }
}

export const getChannelId = (partyA: string, partyB: string) => {
  return keccak256(
    {
      type: 'address',
      value: partyA,
    },
    {
      type: 'address',
      value: partyB,
    }
  )
}

export const encode = (items: { type: string; value: string }[]): string => {
  const { types, values } = items.reduce(
    (result, item) => {
      result.types.push(item.type)
      result.values.push(item.value)

      return result
    },
    {
      types: [],
      values: [],
    }
  )

  return web3.eth.abi.encodeParameters(types, values)
}
