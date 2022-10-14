// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./utils/Accounts.sol";
import "forge-std/Test.sol";

contract HoprChannelsTest is Test, AccountsFixture {
    function setUp() public {}

    // it('should announce user', async function () {
    //   const { channels, deployer, deployerPubKey } = await useFixtures()

    //   const deployerAddress = await deployer.getAddress()
    //   const publicKey = deployerPubKey.toUncompressedPubKeyHex()
    //   await expect(channels.connect(deployer).announce(deployerPubKey.toUncompressedPubKeyHex(), MULTI_ADDR))
    //     .to.emit(channels, 'Announcement')
    //     .withArgs(deployerAddress, publicKey, MULTI_ADDR)
    // })
    function testAnnounceAddressFromPublicKey() public {
        address computedAddr = address(uint160(uint256(keccak256(accountA.publicKey))));
        emit log_bytes(accountA.publicKey);
        emit log_address(computedAddr);
        assertTrue(computedAddr == accountA.accountAddr);
    }
    function testAnnounceAddressFromPrivateKey() public {
        assertTrue(vm.addr(accountA.privateKey) == accountA.accountAddr);
    }
}
