/**
 * Contains various tests that all commands should pass.
 */
import type { Command } from '../utils/command'
import assert from 'assert'

/**
 * @param cmd command to test
 * @param tests a tuple, first element is the query, second is the expected responses (a command may log more than one response)
 */
export function shouldSucceedExecution(
  cmd: Command,
  [query, expectedResponses]: [query: string, expectedResponses: string[]]
) {
  const responses: string[] = []
  it(`should execute query '${query}' and match all given responses`, function (done) {
    cmd.execute((r) => {
      responses.push(r)

      if (responses.length === expectedResponses.length) {
        for (let i = 0; i < expectedResponses.length; i++) {
          const expectedResponse = expectedResponses[i]
          const response = responses[i]

          assert(response.includes(expectedResponse), `response '${response}' should include '${expectedResponse}`)
        }
        done()
      }
    }, query)
  })
}

/**
 * @param cmd command to test
 * @param tests a tuple, first element is the query, second is the expected error
 */
export function shouldFailExecution(cmd: Command, [query, expectedError]: [query: string, expectedError: string]) {
  it(`should fail to execute query '${query}'`, function (done) {
    cmd.execute((response) => {
      if (response.includes(expectedError)) {
        assert.ok(`response '${response}' should include error '${expectedError}`)
        done()
      }
    }, query)
  })
}

/**
 * @param cmd command to test
 * @param query a query which is invalid
 */
export function shouldFailExecutionOnInvalidQuery(cmd: Command, query: string) {
  return shouldFailExecution(cmd, [query, 'Invalid query'])
}

/**
 * @param cmd command to test
 * @param query a query which contains an invalid parameter
 */
export function shouldFailExecutionOnInvalidParam(cmd: Command, query: string) {
  return shouldFailExecution(cmd, [query, 'Invalid param'])
}

/**
 * @param cmd command to test
 * @param query a valid query
 */
export function shouldFailExecutionOnApiError(cmd: Command, query: string) {
  return shouldFailExecution(cmd, [query, 'Failed to'])
}
