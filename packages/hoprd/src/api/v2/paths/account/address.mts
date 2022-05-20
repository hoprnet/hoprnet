/*
    Deprecated endpoint.
    Endpoint "/api/v2/addresses" should be used instead.
*/

import type { Operation } from 'express-openapi'
import { GET as original } from './addresses.mjs'

export const GET: Operation = [original[0].bind()]
GET.apiDoc = JSON.parse(JSON.stringify(original.apiDoc))
GET.apiDoc.deprecated = true
GET.apiDoc.operationId = 'accountGetAddress'
