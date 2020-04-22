const { NODE_SEEDS, BOOTSTRAP_SEEDS } = require("@hoprnet/hopr-demo-seeds");

const accounts = NODE_SEEDS.concat(BOOTSTRAP_SEEDS);
const balance = Number(1000000000000000000000000).toString(16);

module.exports = {
  providerOptions: {
    accounts: accounts.map(account => ({
      secretKey: account,
      balance
    }))
  }
};
