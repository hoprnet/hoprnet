import { Operation } from 'express-openapi'

export const parameters = []

export const GET: Operation = [
  (req, res, _next) => {
    const { node } = req.context
    const nativeAddress = node.getEthereumAddress().toHex()
    const hoprAddress = node.getId().toB58String()

    res.status(200).json({ nativeAddress, hoprAddress })
  }
]

GET.apiDoc = {
  description: 'Get the native and hopr addresses of the account associated with the node',
  tags: ['account'],
  operationId: 'accountGetAddress',
  parameters: [],
  responses: {
    '200': {
      description: 'Returns the native and hopr addresses of the account associated with the node',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/Address'
          }
        }
      }
    }
  }
}
