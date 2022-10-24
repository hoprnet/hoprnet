// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "./utils/Deploy.sol";
import "./utils/Accounts.sol";
import "../src/HoprChannels.sol";
import "forge-std/Test.sol";

contract HoprChannelsTest is Test, ERC1820RegistryFixture, AccountsFixture {
    HoprChannels public hoprChannels;

    /**
     * Emitted on every channel state change.
     */
    event ChannelUpdated(
        address indexed source,
        address indexed destination,
        HoprChannels.Channel newState
    );

    /**
     * Emitted once an account announces.
     */
    event Announcement(
        address indexed account,
        bytes publicKey,
        bytes multiaddr
    );

    /**
     * Emitted once a channel if funded.
     */
    event ChannelFunded(
        address indexed funder,
        address indexed source,
        address indexed destination,
        uint256 amount
    );

    function setUp() public virtual override {
        super.setUp();
        // make vm.addr(1) HoprToken contract
        hoprChannels = new HoprChannels(vm.addr(1), 15);
    }

    function testAnnounceAddressFromPublicKey() public {
        bytes memory multiAddress = hex"1234";
        vm.prank(accountA.accountAddr);
        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit Announcement(accountA.accountAddr, accountA.publicKey, multiAddress);
        hoprChannels.announce(accountA.publicKey, multiAddress);
    }

    function testRevert_AnnouceWrongPublicKey() public {
        bytes memory multiAddress = hex"1234";
        vm.prank(accountB.accountAddr);
        vm.expectRevert("publicKey's address does not match senders");
        hoprChannels.announce(accountA.publicKey, multiAddress);
    }

    function testRevert_AnnouceRandomPublicKey(bytes calldata randomPublicKey) public {
        vm.assume(keccak256(randomPublicKey) != keccak256(accountB.publicKey));
        bytes memory multiAddress = hex"1234";
        vm.prank(accountB.accountAddr);
        vm.expectRevert("publicKey's address does not match senders");
        hoprChannels.announce(randomPublicKey, multiAddress);
    }

    // // it should fail to fund without accountA announcement
    function testRevert_FundChannelMultiWithoutAccountAAnnoucement(uint256 amount1) public {
        amount1 = bound(amount1, 1, 1e36);
        // accountA is not annouced and only accountB is announced
        bytes memory multiAddress = hex"1234";
        vm.prank(accountB.accountAddr);
        hoprChannels.announce(accountB.publicKey, multiAddress);
        vm.prank(address(1));

        vm.expectRevert("source has not announced");
        hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, 0);
    }

    // it should fail to fund without accountB announcement
    function testRevert_FundChannelMultiWithoutAccountBAnnoucement(uint256 amount1) public {
        amount1 = bound(amount1, 1, 1e36);
        // accountB is not annouced and only accountA is announced
        bytes memory multiAddress = hex"1234";
        vm.prank(accountA.accountAddr);
        hoprChannels.announce(accountA.publicKey, multiAddress);
        vm.prank(address(1));

        vm.expectRevert("destination has not announced");
        hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, 0);
    }

    function testFundChannelMulti(uint256 amount1, uint256 amount2) public {
        amount1 = bound(amount1, 1, 1e36);
        amount2 = bound(amount2, 1, 1e36);
        // accountA and accountB are both announced
        bytes memory multiAddress = hex"1234";
        vm.prank(accountA.accountAddr);
        hoprChannels.announce(accountA.publicKey, multiAddress);
        vm.prank(accountB.accountAddr);
        hoprChannels.announce(accountB.publicKey, multiAddress);
        vm.prank(address(1));

        // fund channel for two parties triggers 
        vm.expectEmit(true, true, false, false, address(hoprChannels));
        emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, HoprChannels.Channel(amount1, bytes32(0), 0, 0, HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT, 1, 15));
        vm.expectEmit(true, true, true, true, address(hoprChannels));
        emit ChannelFunded(address(1), accountA.accountAddr, accountB.accountAddr, amount1);
        vm.expectEmit(true, true, false, false, address(hoprChannels));
        emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, HoprChannels.Channel(amount2, bytes32(0), 0, 0, HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT, 1, 15));
        vm.expectEmit(true, true, true, true, address(hoprChannels));
        emit ChannelFunded(address(1), accountB.accountAddr, accountA.accountAddr, amount2);
        // fund channel multi calls token transfer under the hood
        vm.mockCall(
            vm.addr(1),
            abi.encodeWithSelector(bytes4(keccak256("transferFrom(address,address,uint256)"))),
            abi.encode(address(1), address(hoprChannels), amount1 + amount2)
        );
        hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, amount2);
    }
    function testFailFundChannelMulti_SameSourceAndDestination(uint256 amount1, uint256 amount2) public {
        amount1 = bound(amount1, 1, 1e36);
        amount2 = bound(amount2, 1, 1e36);
        // accountA is announced
        bytes memory multiAddress = hex"1234";
        vm.prank(accountA.accountAddr);
        hoprChannels.announce(accountA.publicKey, multiAddress);
        vm.prank(address(1));
        hoprChannels.fundChannelMulti(accountA.accountAddr, accountA.accountAddr, amount1, amount2);
    }
    function testFailFundChannelMulti_FromSourceZero(uint256 amount1, uint256 amount2) public {
        amount1 = bound(amount1, 1, 1e36);
        amount2 = bound(amount2, 1, 1e36);
        // accountA is announced
        bytes memory multiAddress = hex"1234";
        vm.prank(accountA.accountAddr);
        hoprChannels.announce(accountA.publicKey, multiAddress);
        vm.prank(address(1));
        hoprChannels.fundChannelMulti(address(0), accountA.accountAddr, amount1, amount2);
    }
    function testFailFundChannelMulti_ToDestinationZero(uint256 amount1, uint256 amount2) public {
        amount1 = bound(amount1, 1, 1e36);
        amount2 = bound(amount2, 1, 1e36);
        // accountA is announced
        bytes memory multiAddress = hex"1234";
        vm.prank(accountA.accountAddr);
        hoprChannels.announce(accountA.publicKey, multiAddress);
        vm.prank(address(1));
        hoprChannels.fundChannelMulti(accountA.accountAddr, address(0), amount1, amount2);
    }
    function testFailFundChannelMulti_AmountZero() public {
        // both accountA and accountB are announced
        bytes memory multiAddress = hex"1234";
        vm.prank(accountA.accountAddr);
        hoprChannels.announce(accountA.publicKey, multiAddress);
        vm.prank(accountB.accountAddr);
        hoprChannels.announce(accountB.publicKey, multiAddress);
        vm.prank(address(1));
        hoprChannels.fundChannelMulti(accountA.accountAddr, address(0), 0, 0);
    }
}
