// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Test.sol";

contract AccountsFixtureTest is Test {
    struct HoprNodeAccount {
        address accountAddr;
        uint256 privateKey;
        bytes publicKey;
    }

    // string memory mnemonic = "test test test test test test test test test test test junk";

    // uint256 accountA.privateKey = vm.deriveKey(mnemonic, 0);
    HoprNodeAccount accountA = HoprNodeAccount({
        accountAddr: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266,
        privateKey: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80,
        publicKey: hex"8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed753547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5"
    });

    // uint256 accountB.privateKey = vm.deriveKey(mnemonic, 1);
    HoprNodeAccount accountB = HoprNodeAccount({
        accountAddr: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8,
        privateKey: 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d,
        publicKey: hex"ba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0d67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4"
    });

    // uint256 accountC.privateKey = vm.deriveKey(mnemonic, 2);
    HoprNodeAccount accountC = HoprNodeAccount({
        accountAddr: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC,
        privateKey: 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a,
        publicKey: hex"9d9031e97dd78ff8c15aa86939de9b1e791066a0224e331bc962a2099a7b1f0464b8bbafe1535f2301c72c2cb3535b172da30b02686ab0393d348614f157fbdb"
    });
}
