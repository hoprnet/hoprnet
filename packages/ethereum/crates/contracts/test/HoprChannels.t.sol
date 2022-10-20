// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./utils/Deploy.sol";
import "./utils/Accounts.sol";
import "../src/HoprChannels.sol";
import "forge-std/Test.sol";

contract HoprChannelsTest is Test, ERC1820RegistryFixture, AccountsFixture {
    HoprChannels public hoprChannel;

    event Announcement(
        address indexed account,
        bytes publicKey,
        bytes multiaddr
    );

    function setUp() public virtual override {
        super.setUp();
        // FIXME: set a fixture of HOPR token
        hoprChannel = new HoprChannels(vm.addr(1), 15);
    }

    function testAnnounceAddressFromPublicKey() public {
        bytes memory multiAddress = hex"1234";
        vm.prank(accountA.accountAddr);
        vm.expectEmit(true, false, false, true, address(hoprChannel));
        emit Announcement(accountA.accountAddr, accountA.publicKey, multiAddress);
        hoprChannel.announce(accountA.publicKey, multiAddress);
    }

    function testRevert_AnnouceWrongPublicKey() public {
        bytes memory multiAddress = hex"1234";
        vm.prank(accountB.accountAddr);
        vm.expectRevert("publicKey's address does not match senders");
        hoprChannel.announce(accountA.publicKey, multiAddress);
    }

    function testRevert_AnnouceRandomPublicKey(bytes calldata randomPublicKey) public {
        vm.assume(keccak256(randomPublicKey) != keccak256(accountB.publicKey));
        bytes memory multiAddress = hex"1234";
        vm.prank(accountB.accountAddr);
        vm.expectRevert("publicKey's address does not match senders");
        hoprChannel.announce(randomPublicKey, multiAddress);
    }

    // function testFundChannelMulti()
}
