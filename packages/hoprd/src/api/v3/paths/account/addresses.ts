import type { Operation } from 'express-openapi'
import type { Hopr } from '@hoprnet/hopr-core'
import { STATUS_CODES } from '../../utils.js'

/**
 * @returns Native and HOPR addresses of the account associated with the node.
 */
export const getAddresses = (
  node: Hopr
): {
  native: string
  hopr: string
} => {
  const native = node.getEthereumAddress().to_hex()
  const hopr = node.getId().toString()

  return {
    native,
    hopr
  }
}

const GET: Operation = [
  (req, res, _next) => {
    const { node } = req.context

    try {
      const addresses = getAddresses(node)

      return res.status(200).json({
        native: addresses.native,
        hopr: addresses.hopr
      })
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
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
              native: {
                $ref: '#/components/schemas/NativeAddress'
              },
              hopr: {
                $ref: '#/components/schemas/HoprAddress'
              }
            }
          }
        }
      }
    },
    '401': {
      $ref: '#/components/responses/Unauthorized'
    },
    '403': {
      $ref: '#/components/responses/Forbidden'
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

export default { GET }
