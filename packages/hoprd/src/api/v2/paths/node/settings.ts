import { Operation } from "express-openapi"
import { isError } from "../../logic"
import { APIv2Settings, getSetting, setSetting } from "../../logic/settings"

export const GET: Operation = [
    async (req, res, _next) => {
        const { state, node } = req.context
        const { settingName } = req.query

        const setting = getSetting({ node, state, settingName: settingName as keyof APIv2Settings })
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
                                    $ref: "#/components/schemas/Setting"
                                    // type: "object", properties: {
                                    //     name: { type: "string", example: "includeRecipient" }, value: {}
                                    // }
                                },
                                description: 'Setting/s fetched',
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
        },
    }
}

export const POST: Operation = [
    async (req, res, _next) => {
        const { state, node } = req.context
        const { settingName, value } = req.body

        const err = setSetting({ node, state, value, settingName })
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
                    $ref: "#/components/schemas/Setting"
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
