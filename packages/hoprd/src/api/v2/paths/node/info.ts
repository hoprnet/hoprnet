import { Operation } from 'express-openapi'
import { isError } from '../../logic'
import { getInfo } from '../../logic/info'


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
