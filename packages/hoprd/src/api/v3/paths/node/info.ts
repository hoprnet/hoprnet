import type { Hopr } from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'
import { health_to_string } from '@hoprnet/hopr-utils'

/**
 * @returns Information about the HOPR Node, including any options it started with.
 */
export const getInfo = async (node: Hopr) => {
  try {
    const {
      chain,
      hoprTokenAddress,
      hoprChannelsAddress,
      noticePeriodChannelClosure,
      hoprNetworkRegistryAddress,
      hoprNodeSafeRegistryAddress,
      moduleAddress,
      safeAddress
    } = node.smartContractInfo()

    return {
      network: node.network.id,
      announcedAddress: (await node.getAddressesAnnouncedToDHT()).map((ma) => ma.toString()),
      listeningAddress: (await node.getListeningAddresses()).map((ma) => ma.toString()),
      chain: chain,
      hoprToken: hoprTokenAddress,
      hoprChannels: hoprChannelsAddress,
      hoprNetworkRegistry: hoprNetworkRegistryAddress,
      hoprNodeSafeRegistry: hoprNodeSafeRegistryAddress,
      nodeManagementModule: moduleAddress,
      nodeSafe: safeAddress,
      isEligible: await node.isAllowedAccessToNetwork(node.getId()),
      connectivityStatus: health_to_string(await node.getConnectivityHealth()),
      channelClosurePeriod: Math.ceil(noticePeriodChannelClosure / 60)
    }
  } catch (error) {
    // Make sure this doesn't throw
    const errString = error instanceof Error ? error.message : 'Unknown error'
    throw new Error(errString)
  }
}

const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context

    try {
      const info = await getInfo(node)
      return res.status(200).send(info)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description:
    'Information about the HOPR Node, including any options it started with. See the schema of the response to get more information on each field.',
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
              network: {
                type: 'string',
                example: 'anvil-localhost',
                description: 'Name of the network the node is running on.'
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
              chain: {
                type: 'string',
                example: 'anvil',
                description: 'Name of the Hopr network this node connects to.'
              },
              hoprToken: {
                type: 'string',
                example: '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
                description: 'Contract address of the Hopr token on the ethereum chain.'
              },
              hoprChannels: {
                type: 'string',
                example: '0x2a54194c8fe0e3CdeAa39c49B95495aA3b44Db63',
                description:
                  'Contract address of the HoprChannels smart contract on ethereum chain. This smart contract is used to open payment channels between nodes on blockchain.'
              },
              hoprNetworkRegistryAddress: {
                type: 'string',
                example: '0xBEE1F5d64b562715E749771408d06D57EE0892A7',
                description:
                  'Contract address of the contract that allows to control the number of nodes in the network'
              },
              hoprNodeSafeRegistryAddress: {
                type: 'string',
                example: '0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82',
                description: 'Contract address of the contract that register node and safe pairs'
              },
              nodeManagementModule: {
                type: 'string',
                example: '0x0262496080c3916d9afd5904EA3DFd46DfFBfF3D',
                description: 'Contract address of the Safe module for managing the current hopr node'
              },
              nodeSafe: {
                type: 'string',
                example: '0x8D56Ef78c9dfF0d1446A28476653D979336032f9',
                description: 'Contract address of the safe that holds asset for the current node'
              },
              connectivityStatus: {
                type: 'string',
                example: 'GREEN',
                description:
                  'Indicates how good is the connectivity of this node to the HOPR network: either RED, ORANGE, YELLOW or GREEN'
              },
              isEligible: {
                type: 'boolean',
                example: true,
                description:
                  'Determines whether the staking account associated with this node is eligible for accessing the HOPR network. Always true if network registry is disabled.'
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
