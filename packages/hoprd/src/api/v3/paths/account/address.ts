/*
    Deprecated endpoint.
    Endpoint "/api/v2/addresses" should be used instead.
*/

import type { Operation } from 'express-openapi'
import { default as original } from './addresses.js'

const GET: Operation = [original.GET[0].bind({})]
GET.apiDoc = JSON.parse(JSON.stringify(original.GET.apiDoc))
GET.apiDoc.deprecated = true
GET.apiDoc.operationId = 'accountGetAddress'

export default { GET }
