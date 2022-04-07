import type Hopr from '@hoprnet/hopr-core'
import { PassiveStrategy, PromiscuousStrategy } from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils'
import { SettingKey, State, StateOps } from '../../../../types'

/**
 * Sets node setting/s in HOPRd state.
 * Updates HOPRd's state.
 * @returns Setting value or all settings values.
 */
export const setSetting = (node: Hopr, stateOps: StateOps, key: keyof State['settings'], value: any) => {
  const state = stateOps.getState()
  if (!Object.values(SettingKey).includes(key)) {
    throw Error(STATUS_CODES.INVALID_SETTING)
  }

  switch (key) {
    case SettingKey.INCLUDE_RECIPIENT:
      if (typeof value !== 'boolean') throw Error(STATUS_CODES.INVALID_SETTING_VALUE)
      state.settings[key] = value
      break
    case SettingKey.STRATEGY:
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

export const PUT: Operation = [
  async (req, res, _next) => {
    const { stateOps, node } = req.context
    const { setting } = req.params
    const { settingValue } = req.body

    try {
      setSetting(node, stateOps, setting as SettingKey, settingValue)
      return res.status(204).send()
    } catch (error) {
      // Can't validate setting value on express validation level bacause the type of settingValue depends on settingKey.
      // CustomFormats validation check in express-openapi doesn't have access to rest of the request body so we can't check setting key,
      // that's why we leave validation of the setting value to the logic function and not route code.
      if (error.message.includes(STATUS_CODES.INVALID_SETTING_VALUE)) {
        return res.status(400).send({ status: STATUS_CODES.INVALID_SETTING_VALUE })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: error.message })
      }
    }
  }
]

PUT.apiDoc = {
  description: `Change this node's setting value. Check Settings schema to learn more about each setting and the type of value it expects.`,
  tags: ['Settings'],
  operationId: 'settingsSetSetting',
  parameters: [
    {
      in: 'path',
      name: 'setting',
      required: true,
      schema: {
        format: 'settingKey',
        type: 'string',
        description: 'Name of the setting we want to change.',
        example: 'includeRecipient'
      }
    }
  ],
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['settingValue'],
          properties: {
            settingValue: {}
          },
          example: { settingValue: true }
        }
      }
    }
  },
  responses: {
    '204': {
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
