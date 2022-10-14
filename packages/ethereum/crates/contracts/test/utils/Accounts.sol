// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import "forge-std/Test.sol";

contract AccountsFixture is Test {
    struct Account {
        address accountAddr;
        uint256 privateKey;
        bytes publicKey;
    }

    // string memory mnemonic = "test test test test test test test test test test test junk";

    // uint256 accountA.privateKey = vm.deriveKey(mnemonic, 0);
    Account accountA = Account({
        accountAddr: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266,
        privateKey: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80,
        publicKey: hex"8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed753547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5"
    });

    // uint256 accountA.privateKey = vm.deriveKey(mnemonic, 1);
    Account accountB = Account({
        accountAddr: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8,
        privateKey: 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d,
        publicKey: hex"ba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0d67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4"
    });
}
