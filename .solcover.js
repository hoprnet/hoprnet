module.exports = {
    port: 8555,
    copyNodeModules: false,
    // compileCommand: '../node_modules/.bin/truffle compile',
    testrpcOptions: '--port 8555 --defaultBalanceEther 1000000', //-e or --defaultBalanceEther: Amount of ether to assign each test account. Default is 100.
    // testCommand: '../node_modules/.bin/truffle test --network coverage',
    copyPackages: ['openzeppelin-solidity'],
    norpc: false
};

