/*
    Deprecated endpoint.
    Endpoint "/api/v2/addresses" should be used instead.
*/

import { GET } from './addresses'

export { GET }
GET.apiDoc.deprecated = true
GET.apiDoc.operationId = 'accountGetAddress'
