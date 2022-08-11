import assert from 'assert'
import type { Command } from '../utils/command'

/**
 * @param cmd command to test
 * @param tests a tuple, first element is the query, second is the expected responses (a command may log more than one response)
 */
export function shouldSucceedExecution(cmd: Command, tests: [query: string, expectedResponses: string[]][]) {
  for (const [query, expectedResponses] of tests) {
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
}

/**
 * @param cmd command to test
 * @param tests a tuple, first element is the query, second is the expected error
 */
export function shouldFailExecution(cmd: Command, tests: [query: string, expectedError: string][]) {
  for (const [query, expectedError] of tests) {
    it(`should fail to execute query '${query}'`, function (done) {
      cmd.execute((response) => {
        assert(response.includes(expectedError), `response '${response}' should include error '${expectedError}`)
        done()
      }, query)
    })
  }
}

/**
 * @param cmd command to test
 * @param query an incorrect query to run
 */
export function shouldFailExecutionOnIncorrectParam(cmd: Command, query: string) {
  return shouldFailExecution(cmd, [[query, 'Incorrect']])
}

/**
 * @param cmd command to test
 * @param query a query to run, must be valid
 */
export function shouldFailExecutionOnApiError(cmd: Command, query: string) {
  return shouldFailExecution(cmd, [[query, 'Failed to']])
}

/**
 * @param cmd command to test with correct API response
 * @param cmdNoApi command to test with incorrect API response
 * @param successTests
 * @param failureTests
 * @param apiErrorQuery
 * @param incorrectParamQuery
 */
export function shouldBehaveLikeACommand(
  cmd: Command,
  cmdNoApi: Command,
  incorrectParamQuery: string,
  apiErrorQuery: string,
  successTests: [query: string, expectedResponses: string[]][],
  failureTests: [query: string, expectedError: string][]
) {
  shouldFailExecutionOnIncorrectParam(cmd, incorrectParamQuery)
  shouldFailExecutionOnApiError(cmdNoApi, apiErrorQuery)
  shouldSucceedExecution(cmd, successTests)
  shouldFailExecution(cmd, failureTests)
}
