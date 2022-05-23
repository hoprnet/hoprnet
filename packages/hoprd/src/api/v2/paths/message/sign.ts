import type { Operation } from 'express-openapi'
import { POST as original } from '../messages/sign.js'

/*
    Deprecated endpoint.
    Endpoint "/api/v2/messages/sign" should be used instead.
*/
export const POST: Operation = [original[0].bind()]
POST.apiDoc = JSON.parse(JSON.stringify(original.apiDoc))
POST.apiDoc.deprecated = true
POST.apiDoc.operationId = 'messageSign'
