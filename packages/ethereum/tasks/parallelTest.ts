import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import { TASK_TEST, TASK_TEST_GET_TEST_FILES } from 'hardhat/builtin-tasks/task-names'
import { resolve } from 'path'

export type ParallelTestCLIOpts = {
  config: Array<{
    date: string
    testFiles: string[]
  }>
}

/**
 * Put test files into groups that shares the same ganache instances
 */
async function main(opts: ParallelTestCLIOpts, { run, config }: HardhatRuntimeEnvironment): Promise<void> {
  const groupedTestFiles = [] // an array of arrays of relative paths of test files. Test files of one array shares the same hardhat node instance.
  // Test files without specifying their initialDate are saved in this variable. They use the default initialDate (now). This variable is populated with all the test files under the default test folder
  let remainingTestFiles: string[] = await run(TASK_TEST_GET_TEST_FILES)

  // put test files into groups with specified ganach starting dates
  if (opts.config.length > 0) {
    // put test files into groups
    opts.config.forEach((parallelGroup) => {
      const testFiles = parallelGroup.testFiles.map((testFile) => resolve(config.paths.tests, testFile))
      remainingTestFiles = remainingTestFiles.filter((remainingTestFile) => !testFiles.includes(remainingTestFile))
      groupedTestFiles.push(testFiles)
    })
  }
  // leave the remaining test files with the default hardhat config file.
  groupedTestFiles.push([...remainingTestFiles])

  // run tests with their config files
  for (const testFiles of groupedTestFiles) {
    await run(TASK_TEST, { testFiles })
  }
}

export default main
