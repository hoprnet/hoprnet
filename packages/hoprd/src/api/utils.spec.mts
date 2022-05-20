import assert from 'assert'
import { authenticateWsConnection, removeQueryParams } from './utils.mjs'

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

describe('Test authenticateWsConnection', function () {
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

describe('Test removeQueryParams', function () {
  it('should strip away parameters', function () {
    assert.equal(
      removeQueryParams('/api/v2/messages/websocket?apiToken=^^LOCAL-testing-123^^'),
      '/api/v2/messages/websocket'
    )
  })

  it('should strip away parameters and tailing slash', function () {
    assert.equal(
      removeQueryParams('/api/v2/messages/websocket/?apiToken=^^LOCAL-testing-123^^'),
      '/api/v2/messages/websocket'
    )
  })

  it('should strip away single slash', function () {
    assert.equal(removeQueryParams('/?apiToken=^^LOCAL-testing-123^^'), '')
    assert.equal(removeQueryParams('/'), '')
  })
})
