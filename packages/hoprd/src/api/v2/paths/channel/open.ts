import { Operation } from 'express-openapi'

export const openChannel = async ({
  counterpartyPeerId,
  amountToFundStr,
  state,
  node
}: {
  counterpartyPeerId: string
  amountToFundStr: string
  state: State
  node: Hopr
}) => {
  let counterparty: PeerId
  try {
    counterparty = PeerId.createFromB58String(counterpartyPeerId)
  } catch (err) {
    return new Error('invalidPeerId')
  }

  let amountToFund: BN
  let myAvailableTokens: Balance
  try {
    amountToFund = new BN(moveDecimalPoint(amountToFundStr, Balance.DECIMALS))
    myAvailableTokens = await node.getBalance()
  } catch (error) {
    return new Error('invalidAmountToFund')
  }

  if (amountToFund.lten(0)) {
    return new Error('invalidAmountToFund')
  } else if (amountToFund.gt(myAvailableTokens.toBN())) {
    return new Error(
      JSON.stringify({
        status: 'notEnoughFunds',
        tokensRequired: amountToFund.toString(10),
        currentBalance: myAvailableTokens.toBN().toString(10)
      })
    )
  }

  try {
    const { channelId } = await node.openChannel(counterparty, amountToFund)
    return channelId.toHex()
  } catch (err) {
    return new Error(err.message.includes('Channel is already opened') ? 'channelAlreadyOpen' : 'failure')
  }
}

export const parameters = []

export const POST: Operation = [
  async (req, res, _next) => {
    const { node, stateOps } = req.context
    const { peerId, amount } = req.body

    // NOTE: probably express can or already is handling it automatically
    if (!peerId || !amount) {
      return res.status(400).send({ status: 'missingBodyfields' })
    }

    const channelId = await openChannel({
      amountToFundStr: amount,
      counterpartyPeerId: peerId,
      node: node,
      state: stateOps.getState()
    })
    if (isError(channelId)) {
      let errorStatus

      switch (channelId.message) {
        case 'invalidAmountToFund':
        case 'invalidPeerId':
          errorStatus = 400
          break
        case 'channelAlreadyOpen':
          errorStatus = 403
          break
        default:
          errorStatus = 500
      }

      return res
        .status(errorStatus)
        .send(
          channelId.message.includes('notEnoughFunds') ? JSON.parse(channelId.message) : { status: channelId.message }
        )
    } else {
      return res.status(200).send({ status: 'success', channelId })
    }
  }
]

POST.apiDoc = {
  description: 'Opens a payment channel between you and the counter party provided',
  tags: ['channel'],
  operationId: 'openChannel',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            peerId: {
              type: 'string',
              description:
                'peerId that we want to transact with using this channel, in other words a receiver of funds.'
            },
            amount: { type: 'string', description: 'Amount of tokens to fund the channel.' }
          },
          example: {
            peerId: '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12',
            amount: '0.001'
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Channel succesfuly opened',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              channelId: { type: 'string', example: '7b379578588920ca78fbf' }
            }
          }
        }
      }
    },
    '400': {
      description: 'Problem with inputs',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'invalidPeerId | invalidAmountToFund' }
        }
      }
    },
    '403': {
      description: 'Channel already open',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'channelAlreadyOpen' }
        }
      }
    },
    '500': {
      description: 'Insufficient balance to open channel',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'notEnoughFunds' },
              tokensRequired: { type: 'string', example: '10' },
              currentBalance: { type: 'string', example: '9' }
            }
          }
        }
      }
    }
  }
}
