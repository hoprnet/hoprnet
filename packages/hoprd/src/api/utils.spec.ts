import assert from 'assert'
import { authenticateWsConnection } from './utils'

// mocks
const VALID_API_TOKEN = 'VALID_API_TOKEN_123_^^'
const REQ_PARAM = {
  url: `?apiToken=${VALID_API_TOKEN}`,
  headers: {}
}
const REQ_COOKIES = {
  url: '',
  headers: {
    cookie: `X-Auth-Token=${VALID_API_TOKEN}`
  }
}

describe('Test API utils', function () {
  it('should throw on empty API token', function () {
    assert.throws(() => authenticateWsConnection(REQ_PARAM, ''), /Cannot authenticate empty apiToken/)
  })

  it('should authenticate via API token', function () {
    assert(authenticateWsConnection(REQ_PARAM, VALID_API_TOKEN))
  })

  it('should authenticate via API cookie', function () {
    assert(authenticateWsConnection(REQ_COOKIES, VALID_API_TOKEN))
  })

  it('should fail authentication via invalid param', function () {
    assert(
      !authenticateWsConnection(
        {
          url: `?apiToken=invalid`,
          headers: {}
        },
        VALID_API_TOKEN
      )
    )
  })

  it('should fail authentication via invalid cookie', function () {
    assert(
      !authenticateWsConnection(
        {
          url: '',
          headers: {
            cookie: `X-Auth-Token=invalid`
          }
        },
        VALID_API_TOKEN
      )
    )
  })
})
