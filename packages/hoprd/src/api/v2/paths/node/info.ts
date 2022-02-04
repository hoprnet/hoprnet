import Hopr from '@hoprnet/hopr-core'
import { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../'

/**
 * @returns Information about the HOPR Node, including any options it started with.
 */
export const getInfo = async ({ node }: { node: Hopr }) => {
  try {
    const { network, hoprTokenAddress, hoprChannelsAddress, channelClosureSecs } = node.smartContractInfo()

    return {
      environment: node.environment.id,
      announcedAddress: (await node.getAnnouncedAddresses()).map((ma) => ma.toString()),
      listeningAddress: node.getListeningAddresses().map((ma) => ma.toString()),
      network: network,
      hoprToken: hoprTokenAddress,
      hoprChannels: hoprChannelsAddress,
      channelClosurePeriod: Math.ceil(channelClosureSecs / 60)
    }
  } catch (error) {
    throw new Error(STATUS_CODES.UNKNOWN_FAILURE + error.message)
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const info = await getInfo({ node })
      return res.status(200).send(info)
    } catch (error) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: error.message })
    }
  }
]

GET.apiDoc = {
  description:
    'Information about the HOPR Node, including any options it started with. See the schema of the response to get more information on each field',
  tags: ['Node'],
  operationId: 'nodeGetInfo',
  responses: {
    '200': {
      description: 'Node information fetched successfuly.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              environment: {
                type: 'string',
                example: 'hardhat-localhost',
                description: 'Name of the enviroment the node is running on.'
              },
              announcedAddress: {
                type: 'array',
                items: {
                  type: 'string',
                  description:
                    'description: Public IP address that the node announced on network when it was launched. Node anouncing means notifying all the other nodes on the network about its presence and readiness to be connected to via websocket.'
                },
                example: [
                  '/ip4/128.0.215.32/tcp/9080/p2p/16Uiu2HAm91QFjPepnwjuZWzK5pb5ZS8z8qxQRfKZJNXjkgGNUAit',
                  '/p2p/16Uiu2HAmLpqczAGfgmJchVgVk233rmB2T3DSn2gPG6JMa5brEHZ1/p2p-circuit/p2p/16Uiu2HAm91QFjPepnwjuZWzK5pb5ZS8z8qxQRfKZJNXjkgGNUAit',
                  '/ip4/127.0.0.1/tcp/9080/p2p/16Uiu2HAm91QFjPepnwjuZWzK5pb5ZS8z8qxQRfKZJNXjkgGNUAit',
                  '/ip4/192.168.178.56/tcp/9080/p2p/16Uiu2HAm91QFjPepnwjuZWzK5pb5ZS8z8qxQRfKZJNXjkgGNUAit'
                ]
              },
              listeningAddress: {
                type: 'array',
                items: {
                  type: 'string',
                  description: 'Other nodes IP address that this node is listening to for websocket events.'
                },
                example: ['/ip4/0.0.0.0/tcp/9080/p2p/16Uiu2HAm91QFjPepnwjuZWzK5pb5ZS8z8qxQRfKZJNXjkgGNUAit']
              },
              network: {
                type: 'string',
                example: 'hardhat',
                description: 'Name of the Hopr network this node connects to.'
              },
              hoprToken: {
                type: 'string',
                example: '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
                description: 'Contract address of the Hopr token on the ethereum network.'
              },
              hoprChannels: {
                type: 'string',
                example: '0x2a54194c8fe0e3CdeAa39c49B95495aA3b44Db63',
                description:
                  'Contract address of the HoprChannels smart contract on ethereum network. This smart contract is used to open payment channels between nodes on blockchain.'
              },
              channelClosurePeriod: {
                type: 'number',
                example: 1,
                description:
                  'Time (in minutes) that this node needs in order to clean up before closing the channel. When requesting to close the channel each node needs some time to make sure that channel can be closed securely and cleanly. After this channelClosurePeriod passes the second request for closing channel will close the channel.'
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
