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
 * Create config files for tests in "parallel" ganache instances
 */
async function main(opts: ParallelTestCLIOpts, { run, config }: HardhatRuntimeEnvironment): Promise<void> {
  const parallelTestConfigs = [] // an array of hardhat config files. parallelTestConfigs[i] is for groupedTestFiles[i]
  const groupedTestFiles = [] // an array of arrays of relative paths of test files. Test files of one array shares the same config file.
  // Test files without specifying their initialDate are saved in this variable. They use the default initialDate (now). This variable is populated with all the test files under the default test folder
  let remainingTestFiles: string[] = await run(TASK_TEST_GET_TEST_FILES)

  // put test files into groups with specified ganach starting dates
  if (opts.config.length > 0) {
    // put test files into groups
    opts.config.forEach((parallelGroup) => {
      const testFiles = parallelGroup.testFiles.map((testFile) => resolve(config.paths.tests, testFile))
      remainingTestFiles = remainingTestFiles.filter((remainingTestFile) => !testFiles.includes(remainingTestFile))
      const newConfig = { ...config }
      newConfig.networks.hardhat.initialDate = parallelGroup.date
      parallelTestConfigs.push(newConfig)
      groupedTestFiles.push(testFiles)
    })
  }
  // leave the remaining test files with the default hardhat config file.
  parallelTestConfigs.push({ ...config })
  groupedTestFiles.push([...remainingTestFiles])

  // run tests with their config files
  await run(TASK_TEST, { testFiles: groupedTestFiles[0] })
  await run(TASK_TEST, { testFiles: groupedTestFiles[1] })
}

export default main
