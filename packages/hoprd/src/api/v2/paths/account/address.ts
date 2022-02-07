/*
    Deprecated endpoint.
    Endpoint "/api/v2/addresses" should be used instead.
*/

import { Operation } from 'express-openapi'
import { GET as original } from './addresses'

export const GET: Operation = [original[0].bind()]
GET.apiDoc = JSON.parse(JSON.stringify(original.apiDoc))
GET.apiDoc.deprecated = true
GET.apiDoc.operationId = 'accountGetAddress'
