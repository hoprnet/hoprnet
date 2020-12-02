const parseEnvs = (stdout) => stdout.split('\n').map((line) => line.length > 0 && line.split('='))

const parseStdout = (stdout, envMap) =>
  parseEnvs(stdout)
    .filter(Boolean)
    .map((envs) => (([env, value]) => (envMap[env] = value))(envs))

const parsePromises = (envMap) => (envs) => envs.forEach(({ stdout, stderr }) => parseStdout(stdout, envMap))

module.exports = { parseEnvs, parseStdout, parsePromises }
