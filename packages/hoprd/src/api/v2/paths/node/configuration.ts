import Hopr from '@hoprnet/hopr-core'
import { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'

/**
 * @returns Configuration about the HOPR Node.
 */
export const getConfiguration = async ({ node }: { node: Hopr }) => {
  try {
    const { environment, network } = node.getPublicHoprOptions()
    const { hoprTokenAddress, hoprChannelsAddress, channelClosureSecs, hoprNetworkRegistryAddress } =
      node.smartContractInfo()
    const channelClosureMins = Math.ceil(channelClosureSecs / 60) // convert to minutes

    return {
      environment: environment,
      network: network,
      hoprToken: hoprTokenAddress,
      hoprChannels: hoprChannelsAddress,
      hoprNetworkRegistry: hoprNetworkRegistryAddress,
      isEligible: await node.isAllowedAccessToNetwork(node.getId()),
      channelClosurePeriod: channelClosureMins
    }
  } catch (error) {
    // Make sure this doesn't throw
    const errString = error instanceof Error ? error.message : 'Unknown error'
    throw new Error(errString)
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const configuration = await getConfiguration({ node })
      return res.status(200).send(configuration)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Configuration of the HOPR Node. See the schema of the response to get more information on each field.',
  tags: ['Node'],
  operationId: 'nodeGetConfiguration',
  responses: {
    '200': {
      description: 'Node configuration fetched successfuly.',
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
              hoprNetworkRegistryAddress: {
                type: 'string',
                example: '0xBEE1F5d64b562715E749771408d06D57EE0892A7',
                description:
                  'Contract address of the contract that allows to control the number of nodes in the network'
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
