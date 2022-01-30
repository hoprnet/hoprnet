import type { Operation } from 'express-openapi'
import type Hopr from '@hoprnet/hopr-core'

export const getAddresses = (
  node: Hopr
): {
  native: string
  hopr: string
} => {
  const native = node.getEthereumAddress().toHex()
  const hopr = node.getId().toB58String()

  return {
    native,
    hopr
  }
}

export const GET: Operation = [
  (req, res, _next) => {
    const { node } = req.context
    const addresses = getAddresses(node)

    res.status(200).json({
      nativeBalance: addresses.native,
      hoprBalance: addresses.hopr
    })
  }
]

GET.apiDoc = {
  description: 'Get the native and hopr addresses of the account associated with the node.',
  tags: ['account'],
  operationId: 'accountGetAddress',
  parameters: [],
  responses: {
    '200': {
      description: 'Returns the native and hopr addresses of the account associated with the node.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/Address'
          }
        }
      }
    }
  }
}
