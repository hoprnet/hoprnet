const setupEnv = require('./set-up-env');

const main = async () => {
  const env = await setupEnv();
  console.log("env", env)
}

main();