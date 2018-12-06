const sha3      = require('web3-utils').sha3;
const fs        = require('fs');
const assert    = require('assert');

// Valid hashes using Keccak-256

const contracts = {
    Crowdsale       : fs.readFileSync('node_modules/openzeppelin-solidity/contracts/crowdsale/Crowdsale.sol'),
    ERC20Mintable   : fs.readFileSync('node_modules/openzeppelin-solidity/contracts/token/ERC20/ERC20Mintable.sol'),
    ERC20Pausable   : fs.readFileSync('node_modules/openzeppelin-solidity/contracts/token/ERC20/ERC20Pausable.sol'),
    Pausable        : fs.readFileSync('node_modules/openzeppelin-solidity/contracts/lifecycle/Pausable.sol'),
    Ownable         : fs.readFileSync('node_modules/openzeppelin-solidity/contracts/ownership/Ownable.sol'),
    ERC20           : fs.readFileSync('node_modules/openzeppelin-solidity/contracts/token/ERC20/ERC20.sol'),
    IERC20          : fs.readFileSync('node_modules/openzeppelin-solidity/contracts/token/ERC20/IERC20.sol'),
    SafeMath        : fs.readFileSync('node_modules/openzeppelin-solidity/contracts/math/SafeMath.sol'),
    TokenVesting    : fs.readFileSync('node_modules/openzeppelin-solidity/contracts/drafts/TokenVesting.sol')
};

const hashes = {
    Crowdsale     : '0x4374c1201f1c67473983736b00560ec936a7a1509921251d17e1a2f2fba10ea2',
    ERC20Mintable : '0x9f78ffbde6b7ea69b440136d5bdec213eac31de62a7689e8f6eed5cf3d1ebf4e',
    ERC20Pausable : '0x34ae26a75fc5dd5d3d283d0d95a080212e81734a5cab7be187778e74224d57d1',
    Pausable      : '0x9cb69bde8cc3efdd07a2c372e78b664e961f505b3e4195ee27f06b358ffef996',
    Ownable       : '0x973e50d102a48ea6e132f18793b096910ed508656e89706744112cbb4ecaa9dd',
    ERC20         : '0xaa2cf35177f37a9af9c975a973c813c5520be56ccee3a39b6c9883d4781cd072',
    IERC20        : '0xc8a7f15fa44eb10057f8bcb58418d3aed6f5267d211d18f93f3d4e0293e3da44',
    SafeMath      : '0xdd18f05c74536a06166c90a17ec3c6932853764f292bed314ceb23e730ff83af',
    TokenVesting  : '0xb713357d754cc2e44c28bfd8d21f39bdd7238d83fe07a12a0f5a89289d04b040'
};

Object.keys(contracts).forEach((key) => {
    try {
        assert.equal(sha3(contracts[key]), hashes[key], 'Hash mismatch: ' + key);
    } catch (error) {
        console.log(error.message + ' - Zeppelin Framework');
        console.log(key + ': ' + sha3(contracts[key]));
    }
});
