import type Hopr from '@hoprnet/hopr-core'
import { PassiveStrategy, PromiscuousStrategy } from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../'
import type { State, StateOps } from '../../../../types'

export interface Setting {
  name: string
  value: any
}

/**
 * Reads node setting/s from HOPRd state.
 * @returns Setting value or all settings values.
 */
export const getSetting = ({
  node,
  state,
  settingName
}: {
  node: Hopr
  state: State
  settingName?: keyof State['settings']
}) => {
  const getSettingByName = (name: string): Setting => {
    if (name) {
      const setting = state.settings[name]

      if (typeof setting === 'undefined') {
        throw Error(STATUS_CODES.INVALID_SETTING)
      }

      if (name === 'strategy') {
        return { name, value: node.getChannelStrategy() }
      } else {
        return { name, value: setting }
      }
    }
  }

  if (!settingName) {
    const settingsNames: (keyof State['settings'])[] = ['includeRecipient', 'strategy']
    return settingsNames.map(getSettingByName)
  }

  return getSettingByName(settingName)
}

/**
 * Sets node setting/s in HOPRd state.
 * Updates HOPRd's state.
 * @returns Setting value or all settings values.
 */
export const setSetting = ({
  node,
  settingName,
  stateOps,
  value
}: {
  settingName: keyof State['settings']
  value: any
  node: Hopr
  stateOps: StateOps
}) => {
  const state = stateOps.getState()
  if (typeof state.settings[settingName] === 'undefined') {
    throw Error(STATUS_CODES.INVALID_SETTING)
  }

  switch (settingName) {
    case 'includeRecipient':
      if (typeof value !== 'boolean') throw Error(STATUS_CODES.INVALID_SETTING_VALUE)
      state.settings[settingName] = value
      break
    case 'strategy':
      let strategy

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
      state.settings[settingName] = value
      break
  }

  stateOps.setState(state)
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps, node } = req.context
    const { settingName } = req.query

    try {
      const setting = getSetting({
        node,
        state: stateOps.getState(),
        settingName: settingName as keyof State['settings']
      })
      return res.status(200).send({ settings: Array.isArray(setting) ? setting : [setting] })
    } catch (error) {
      if (error.message.includes(STATUS_CODES.INVALID_SETTING)) {
        return res.status(400).send({ status: STATUS_CODES.INVALID_SETTING })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: error.message })
      }
    }
  }
]

GET.apiDoc = {
  description: `Get this node's specified setting value.`,
  tags: ['Settings'],
  operationId: 'getSetting',
  parameters: [
    {
      name: 'settingName',
      in: 'query',
      description: 'Setting name that we want to fetch value for.',
      schema: {
        type: 'string',
        example: 'includeRecipient'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Setting fetched succesfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              settings: {
                type: 'array',
                items: {
                  $ref: '#/components/schemas/Setting'
                },
                description: 'Setting/s fetched'
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid input. Setting with that name does not exist.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: STATUS_CODES.INVALID_SETTING }
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

export const POST: Operation = [
  async (req, res, _next) => {
    const { stateOps, node } = req.context
    const { settingName, value } = req.body

    try {
      setSetting({ node, stateOps, value, settingName })
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
  operationId: 'setSetting',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          $ref: '#/components/schemas/Setting'
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
            $ref: '#/components/schemas/StatusResponse'
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
