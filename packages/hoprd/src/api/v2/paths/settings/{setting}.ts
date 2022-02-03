import type Hopr from '@hoprnet/hopr-core'
import { PassiveStrategy, PromiscuousStrategy } from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../'
import type { State, StateOps } from '../../../../types'

/**
 * Sets node setting/s in HOPRd state.
 * Updates HOPRd's state.
 * @returns Setting value or all settings values.
 */
export const setSetting = (node: Hopr, stateOps: StateOps, key: keyof State['settings'], value: any) => {
  const state = stateOps.getState()
  if (typeof state.settings[key] === 'undefined') {
    throw Error(STATUS_CODES.INVALID_SETTING)
  }

  switch (key) {
    case 'includeRecipient':
      if (typeof value !== 'boolean') throw Error(STATUS_CODES.INVALID_SETTING_VALUE)
      state.settings[key] = value
      break
    case 'strategy':
      let strategy: PassiveStrategy | PromiscuousStrategy

      switch (value) {
        case 'passive':
          strategy = new PassiveStrategy()
          break
        case 'promiscuous':
          strategy = new PromiscuousStrategy()
          break
      }
      if (!strategy) throw Error(STATUS_CODES.INVALID_SETTING_VALUE)
      node.setChannelStrategy(strategy)
      state.settings[key] = value
      break
  }

  stateOps.setState(state)
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { stateOps, node } = req.context
    const settings = req.body

    try {
      for (const { key, value } of settings) {
        setSetting(node, stateOps, key, value)
      }
      return res.status(200).send()
    } catch (error) {
      const INVALID_ARG = [STATUS_CODES.INVALID_SETTING_VALUE, STATUS_CODES.INVALID_SETTING].find((arg) =>
        error.message.includes(arg)
      )
      if (INVALID_ARG) {
        return res.status(400).send({ STATUS: INVALID_ARG })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: error.message })
      }
    }
  }
]

POST.apiDoc = {
  description: `Change this node's setting value.`,
  tags: ['Settings'],
  operationId: 'settingsSetSetting',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            key: {
              type: 'string'
            },
            value: {}
          },
          example: { key: 'includeRecipient', value: true }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Setting set succesfully'
    },
    '400': {
      description: `Invalid input. Either setting with that name doesn't exist or the value is incorrect.`,
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: `${STATUS_CODES.INVALID_SETTING} | ${STATUS_CODES.INVALID_SETTING_VALUE}`
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
