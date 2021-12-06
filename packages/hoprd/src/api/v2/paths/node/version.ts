import { Operation } from 'express-openapi'

export const parameters = []

export const GET: Operation = [
  (req, res, _next) => {
    const version = req.context.node.getVersion()
    res.status(200).json({ version })
  }
]

GET.apiDoc = {
  description: 'Get release version of the running node',
  tags: ['node'],
  operationId: 'nodeGetVersion',
  parameters: [],
  responses: {
    '200': {
      description: 'Returns the release version of the running node',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/Version'
          }
        }
      }
    }
  }
}
