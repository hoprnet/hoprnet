const util = require('util');
const execFile = util.promisify(require('child_process').execFile);

const releaseEnvScript = './scripts/cd/release_env.sh'
const addressEnvScript = './scripts/cd/gcloud_env.sh'

const parseEnvs = (stdout) => 
  stdout.split('\n').map(line => 
    line.length > 0 && line.split('='))

const parseStdout = (stdout, envMap) => 
  parseEnvs(stdout)
  .filter(Boolean)
  .map(envs => (([env, value]) => envMap[env] = value)(envs))

const main = async () => {
  try {
    const envMap = {}
    const envPromises = [
      execFile(releaseEnvScript),
      execFile(addressEnvScript)
    ]
    const resolvedEnvPromises = await Promise.all(envPromises)
    resolvedEnvPromises.map(
      ({stdout, stderr}) => parseStdout(stdout, envMap)
    )
    console.log(envMap)
  } catch (e) {
    console.error('Error', e)
  }
}

main()

