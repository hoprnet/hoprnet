import type Hopr from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import type { State } from '../../../../types'

export interface Setting {
  name: string
  value: any
}

export const getSettings = (state: State) => {
  return state.settings
}

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
        throw Error('invalidSetting')
      }

      if (name === 'strategy') {
        return { name, value: node.getChannelStrategy() }
      } else {
        return { name, value: setting }
      }
    }
  }

  if (!settingName) {
  }
  return getSettingByName(settingName)
}

export const setSetting = ({
  node,
  settingName,
  state,
  value
}: {
  settingName: keyof State['settings']
  value: any
  node: Hopr
  state: State
}) => {
  if (typeof state.settings[settingName] === 'undefined') {
    throw Error('invalidSettingName')
  }

  switch (settingName) {
    case 'includeRecipient':
      if (typeof value !== 'boolean') throw Error('invalidValue')
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
      if (!strategy) throw Error('invalidValue')
      node.setChannelStrategy(strategy)
      state.settings[settingName] = value
      break
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps, node } = req.context
    const { settingName } = req.query

    const setting = getSetting({
      node,
      state: stateOps.getState(),
      settingName: settingName as keyof State['settings']
    })
    if (isError(setting)) {
      return res.status(400).send({ status: setting.message })
    } else {
      return res.status(200).send({ status: 'success', settings: Array.isArray(setting) ? setting : [setting] })
    }
  }
]

GET.apiDoc = {
  description: 'Get setting value',
  tags: ['node'],
  operationId: 'getSetting',
  parameters: [
    {
      name: 'settingName',
      in: 'query',
      description: 'Setting name that we want to fetch value for',
      required: true,
      schema: {
        type: 'string',
        example: 'includeRecipient'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Alias/es fetched succesfully',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              settings: {
                type: 'array',
                items: {
                  $ref: '#/components/schemas/Setting'
                  // type: "object", properties: {
                  //     name: { type: "string", example: "includeRecipient" }, value: {}
                  // }
                },
                description: 'Setting/s fetched'
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid input',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'invalidSettingName' }
        }
      }
    }
  }
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { stateOps, node } = req.context
    const { settingName, value } = req.body

    const err = setSetting({ node, state: stateOps.getState(), value, settingName })
    if (isError(err)) {
      return res.status(400).send({ status: err.message })
    } else {
      return res.status(200).send({ status: 'success' })
    }
  }
]

POST.apiDoc = {
  description: 'Change setting value',
  tags: ['node'],
  operationId: 'setSetting',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          $ref: '#/components/schemas/Setting'
          // type: 'object',
          // properties: {
          //     settingName: { type: 'string', description: 'PeerId that we want to set alias to.' },
          //     value: { description: 'Alias that we want to attach to peerId.' }
          // },
          // example: {
          //     settingName: "includeRecipient",
          //     value: true
          // }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Setting set succesfully',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          }
        }
      }
    },
    '400': {
      description: 'Invalid input',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: {
            status: 'invalidSettingName | invalidValue'
          }
        }
      }
    }
  }
}
