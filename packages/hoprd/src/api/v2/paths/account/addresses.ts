import type { Operation } from 'express-openapi'
import type Hopr from '@hoprnet/hopr-core'
import { STATUS_CODES } from '../../'

/**
 * @returns Native and HOPR addresses of the account associated with the node.
 */
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

    try {
      const addresses = getAddresses(node)

      res.status(200).json({
        nativeAddress: addresses.native,
        hoprAddress: addresses.hopr
      })
    } catch (error) {
      res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: error.message })
    }
  }
]

GET.apiDoc = {
  description:
    "Get node's HOPR and native addresses. HOPR address is also called PeerId and can be used by other node owner to interact with this node.",
  tags: ['Account'],
  operationId: 'accountGetAddresses',
  responses: {
    '200': {
      description: 'Addresses fetched successfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              nativeAddress: {
                $ref: '#/components/schemas/NativeAddress'
              },
              hoprAddress: {
                $ref: '#/components/schemas/HoprAddress'
              }
            }
          }
        }
      }
    },
    '422': {
      description: 'Unknown failure.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: STATUS_CODES.UNKNOWN_FAILURE },
              error: { type: 'string', example: 'Full error message.' }
            }
          },
          example: { status: STATUS_CODES.UNKNOWN_FAILURE, error: 'Full error message.' }
        }
      }
    }
  }
}
