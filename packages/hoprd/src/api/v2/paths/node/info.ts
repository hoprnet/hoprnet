import { Operation } from 'express-openapi'
import { isError } from '../..'
import { getInfo } from '../../logic/info'

export const getInfo = async ({ node }: { node: Hopr }) => {
  try {
    const { network, hoprTokenAddress, hoprChannelsAddress, channelClosureSecs } = node.smartContractInfo()

    return {
      amouncedAddress: (await node.getAnnouncedAddresses()).map((ma) => ma.toString()),
      listeningAddress: node.getListeningAddresses().map((ma) => ma.toString()),
      network: network,
      hoprToken: hoprTokenAddress,
      hoprChannels: hoprChannelsAddress,
      channelClosurePeriod: Math.ceil(channelClosureSecs / 60)
    }
  } catch (error) {
    return new Error('failure')
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    const info = await getInfo({ node })
    if (isError(info)) {
      return res.status(500).send({ status: info.message })
    } else {
      return res.status(200).send({ status: 'success', info })
    }
  }
]

GET.apiDoc = {
  description: 'Information about the HOPR Node, including any options it started with',
  tags: ['node'],
  operationId: 'getInfo',
  parameters: [],
  responses: {
    '200': {
      description: 'Info fetched successfuly',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              info: {
                $ref: '#/components/schemas/Info'
              }
            }
          }
        }
      }
    },
    '500': {
      description: 'Failed to get Info.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'failure' }
        }
      }
    }
  }
}
