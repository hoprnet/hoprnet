// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { Test } from "forge-std/Test.sol";

import { ERC1820RegistryFixtureTest } from "./utils/ERC1820Registry.sol";
import { BASE_INDEX_OFFSET, HoprChannels, HoprChannelsType, HoprChannelsEvents } from "../src/Channels.sol";
import { HoprLedgerEvents } from "../src/Ledger.sol";
import { CryptoUtils } from "./utils/Crypto.sol";
import { HoprMultiSig } from "../src/MultiSig.sol";
import { ERC777 } from "../src/static/openzeppelin-contracts/ERC777.sol";
import { HoprNodeSafeRegistry } from "../src/node-stake/NodeSafeRegistry.sol";
import { HoprCrypto } from "../src/Crypto.sol";

// proxy contract to make modifiers testable and manipulate storage
contract MyHoprChannels is HoprChannels {
    constructor(
        address _token,
        Timestamp _noticePeriodChannelClosure,
        HoprNodeSafeRegistry safeRegistry
    )
        HoprChannels(_token, _noticePeriodChannelClosure, safeRegistry)
    { }

    // Only for testing
    function _storeChannelStatus(address src, address dest, HoprChannelsType.ChannelStatus status) public {
        channels[_getChannelId(src, dest)] = HoprChannelsType.Channel(
            HoprChannelsType.Balance.wrap(0),
            HoprChannelsType.TicketIndex.wrap(0),
            HoprChannelsType.Timestamp.wrap(0),
            HoprChannelsType.ChannelEpoch.wrap(0),
            status
        );
    }

    // Only for testing
    function _storeChannel(
        address src,
        address dest,
        uint256 balance,
        uint256 ticketIndex,
        uint256 closureTime,
        uint256 epoch,
        HoprChannelsType.ChannelStatus status
    )
        public
    {
        channels[_getChannelId(src, dest)] = HoprChannelsType.Channel(
            HoprChannelsType.Balance.wrap(uint96(balance)),
            HoprChannelsType.TicketIndex.wrap(uint48(ticketIndex)),
            HoprChannelsType.Timestamp.wrap(uint32(closureTime)),
            HoprChannelsType.ChannelEpoch.wrap(uint24(epoch)),
            status
        );
    }

    // Only for testing
    function _removeChannel(address src, address dest) public {
        delete channels[_getChannelId(src, dest)];
    }

    function myValidateBalance(HoprChannelsType.Balance balance) public validateBalance(balance) { }

    function myValidateChannelParties(
        address source,
        address destination
    )
        public
        validateChannelParties(source, destination)
    { }
}

contract HoprChannelsTest is Test, ERC1820RegistryFixtureTest, CryptoUtils, HoprChannelsEvents, HoprLedgerEvents {
    HoprChannelsType.Timestamp constant CLOSURE_NOTICE_PERIOD = HoprChannelsType.Timestamp.wrap(15);

    bytes32 constant PROOF_OF_RELAY_SECRET_0 = keccak256(abi.encodePacked("PROOF_OF_RELAY_SECRET_0"));
    HoprChannelsType.WinProb constant WIN_PROB_100 = HoprChannelsType.WinProb.wrap(type(uint56).max);
    HoprChannelsType.WinProb constant WIN_PROB_0 = HoprChannelsType.WinProb.wrap(type(uint56).min);

    // We can't use HoprToken because HoprToken and HoprChannels rely
    // on different versions of OpenZeppelin contracts which leads
    // to compilation errors.
    ERC777 hoprToken;

    MyHoprChannels public hoprChannels;
    HoprNodeSafeRegistry public hoprNodeSafeRegistry;

    /// forge-lint:disable-next-line(mixed-case-variable)
    uint256 MIN_USED_BALANCE;
    /// forge-lint:disable-next-line(mixed-case-variable)
    uint256 MAX_USED_BALANCE;

    function setUp() public virtual override {
        super.setUp();

        hoprToken = new ERC777("HOPR", "HOPR", new address[](0));

        hoprNodeSafeRegistry = new HoprNodeSafeRegistry();
        hoprChannels = new MyHoprChannels(address(hoprToken), CLOSURE_NOTICE_PERIOD, hoprNodeSafeRegistry);

        MIN_USED_BALANCE = HoprChannelsType.Balance.unwrap(hoprChannels.MIN_USED_BALANCE()) + 1;
        MAX_USED_BALANCE = HoprChannelsType.Balance.unwrap(hoprChannels.MAX_USED_BALANCE());
    }

    /**
     * @dev test invariants of `Channels.sol` contract
     */
    function test_Constants() public {
        // minimum balance accepted in channel funding
        assertEq(HoprChannelsType.Balance.unwrap(hoprChannels.MIN_USED_BALANCE()), 1);
        // Maximum balance accepted in channel funding
        assertEq(HoprChannelsType.Balance.unwrap(hoprChannels.MAX_USED_BALANCE()), 10 **25);
        // current veersioning
        string memory currentVersiosn = "2.0.0";
        assertEq(hoprChannels.VERSION(), currentVersiosn);
        // acceptable data payload size in ERC777 token send hook
        assertEq(hoprChannels.ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE(), 64); // 20 + 96/8 + 20 + 96/8 = 64
        assertEq(hoprChannels.ERC777_HOOK_FUND_CHANNEL_SIZE(), 40); // 20 + 20
    }

    function testRevert_ConstructorError() public {
        HoprChannels failedChannel;
        // whwen notice period is too short
        vm.expectRevert(HoprChannels.InvalidNoticePeriod.selector);
        failedChannel = new HoprChannels(address(hoprToken), HoprChannelsType.Timestamp.wrap(0), hoprNodeSafeRegistry);

        // when token address is zero
        vm.expectRevert(abi.encodeWithSelector(HoprChannels.ZeroAddress.selector, "_token must not be empty"));
        failedChannel = new HoprChannels(address(0), CLOSURE_NOTICE_PERIOD, hoprNodeSafeRegistry);

        // when NodeSafeRegistry contract is zero
        vm.expectRevert(abi.encodeWithSelector(HoprChannels.ZeroAddress.selector, "_safeRegistry must not be empty"));
        failedChannel = new HoprChannels(address(hoprToken), CLOSURE_NOTICE_PERIOD, HoprNodeSafeRegistry(address(0)));
        vm.clearMockedCalls();
    }

    function test_ViewCurrentBlockTimestamp() public {
        uint256 mockTimestamp = 8765;
        vm.warp(mockTimestamp);

        HoprChannelsType.Timestamp currentTime = hoprChannels._currentBlockTimestamp();
        assertEq(uint256(HoprChannelsType.Timestamp.unwrap(currentTime)), mockTimestamp);
        vm.clearMockedCalls();
    }

    function test_publicFunctions(
        address src,
        address dest,
        uint96 balance,
        uint48 ticketIndex,
        uint32 closureTime,
        uint24 epoch
    )
        public
    {
        assertEq(
            HoprChannelsType.Timestamp.unwrap(hoprChannels.NOTICE_PERIOD_CHANNEL_CLOSURE()),
            HoprChannelsType.Timestamp.unwrap(CLOSURE_NOTICE_PERIOD)
        );
        assertEq(address(hoprChannels.TOKEN()), address(hoprToken));

        hoprChannels._storeChannel(src, dest, balance, ticketIndex, closureTime, epoch, HoprChannelsType.ChannelStatus.OPEN);

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(
                abi.encode(_wrapChannel(balance, ticketIndex, closureTime, epoch, HoprChannelsType.ChannelStatus.OPEN))
            )
        );

        hoprChannels._storeChannel(
            src, dest, balance, ticketIndex, closureTime, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
        );

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(
                abi.encode(
                    _wrapChannel(balance, ticketIndex, closureTime, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE)
                )
            )
        );

        assertEq(hoprChannels.ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE(), 64);
        assertEq(hoprChannels.ERC777_HOOK_FUND_CHANNEL_SIZE(), 40);

        assertEq(hoprChannels.VERSION(), "2.0.0");

        // very unlikely that domainSeparator == bytes32(0)
        assertTrue(hoprChannels.domainSeparator() != bytes32(0));

        // correctly implement ERC-1820
        assertEq(hoprChannels.TOKENS_RECIPIENT_INTERFACE_HASH(), keccak256("ERC777TokensRecipient"));
    }

    function testValidateBalance(uint96 amount) public {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));

        hoprChannels.myValidateBalance(HoprChannelsType.Balance.wrap(amount));
    }

    function testRevert_validateBalance() public {
        vm.expectRevert(abi.encodeWithSelector(HoprChannels.InvalidBalance.selector));
        hoprChannels.myValidateBalance(HoprChannelsType.Balance.wrap(0));

        vm.expectRevert(abi.encodeWithSelector(HoprChannels.BalanceExceedsGlobalPerChannelAllowance.selector));
        hoprChannels.myValidateBalance(HoprChannelsType.Balance.wrap(uint96(MAX_USED_BALANCE) + 1));
    }

    function testValidateChannelParties(address source, address destination) public {
        vm.assume(source != destination);
        vm.assume(source != address(0) && destination != address(0));

        hoprChannels.myValidateChannelParties(source, destination);
    }

    function testRevert_validateChannelParties(address addr) public {
        vm.assume(addr != address(0));

        vm.expectRevert(abi.encodeWithSelector(HoprChannels.SourceEqualsDestination.selector));
        hoprChannels.myValidateChannelParties(addr, addr);

        vm.expectRevert(abi.encodeWithSelector(HoprChannels.ZeroAddress.selector, "source must not be empty"));
        hoprChannels.myValidateChannelParties(address(0), addr);

        vm.expectRevert(abi.encodeWithSelector(HoprChannels.ZeroAddress.selector, "destination must not be empty"));
        hoprChannels.myValidateChannelParties(addr, address(0));
    }

    function test_fundChannel(uint96 amount1, uint96 amount2, address src, address dest, address safeContract) public {
        amount1 = uint96(bound(amount1, MIN_USED_BALANCE, MAX_USED_BALANCE - MIN_USED_BALANCE));
        amount2 = uint96(bound(amount2, MIN_USED_BALANCE, MAX_USED_BALANCE - amount1));
        vm.assume(src != dest && safeContract != src && safeContract != dest);
        vm.assume(src != address(0) && dest != address(0) && safeContract != address(0));

        _helperNoSafeSetMock(src);
        _helperTokenTransferFromMock(src, amount1);

        bytes32 channelId = keccak256(abi.encodePacked(src, dest));
        assertEq(hoprChannels._getChannelId(src, dest), channelId);

        vm.expectEmit(true, true, true, true, address(hoprChannels));
        emit ChannelOpened(channelId, src, dest, _unwrapChannel(HoprChannelsType.Channel(
            HoprChannelsType.Balance.wrap(uint96(amount1)),
            HoprChannelsType.TicketIndex.wrap(uint48(0)),
            HoprChannelsType.Timestamp.wrap(uint32(0)),
            HoprChannelsType.ChannelEpoch.wrap(uint24(1)),
            HoprChannelsType.ChannelStatus.OPEN
        )));

        vm.startPrank(src);
        hoprChannels.fundChannel(dest, HoprChannelsType.Balance.wrap(amount1));

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_wrapChannel(amount1, 0, 0, 1, HoprChannelsType.ChannelStatus.OPEN)))
        );
        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_expandChannel(hoprChannels.channelState(channelId))))
        );

        vm.clearMockedCalls();

        // Now, let's increase funds
        _helperNoSafeSetMock(src);
        _helperTokenTransferFromMock(src, amount2);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelBalanceIncreased(
            channelId, _unwrapChannel(HoprChannelsType.Channel(
                HoprChannelsType.Balance.wrap(uint96(amount1 + amount2)),
                HoprChannelsType.TicketIndex.wrap(uint48(0)),
                HoprChannelsType.Timestamp.wrap(uint32(0)),
                HoprChannelsType.ChannelEpoch.wrap(uint24(1)),
                HoprChannelsType.ChannelStatus.OPEN
            ))
        );

        hoprChannels.fundChannel(dest, HoprChannelsType.Balance.wrap(amount2));

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_wrapChannel(amount1 + amount2, 0, 0, 1, HoprChannelsType.ChannelStatus.OPEN)))
        );
        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_expandChannel(hoprChannels.channelState(channelId))))
        );

        vm.clearMockedCalls();
        vm.stopPrank();

        // clear state
        hoprChannels._removeChannel(src, dest);

        // Now test Safe entry point
        _helperOnlySafeMock(src, safeContract);
        _helperTokenTransferFromMock(safeContract, amount1);

        vm.expectEmit(true, true, true, true, address(hoprChannels));
        emit ChannelOpened(channelId, src, dest, _unwrapChannel(HoprChannelsType.Channel(
                HoprChannelsType.Balance.wrap(uint96(amount1)),
                HoprChannelsType.TicketIndex.wrap(uint48(0)),
                HoprChannelsType.Timestamp.wrap(uint32(0)),
                HoprChannelsType.ChannelEpoch.wrap(uint24(1)),
                HoprChannelsType.ChannelStatus.OPEN
            ))
        );

        vm.startPrank(safeContract);
        hoprChannels.fundChannelSafe(src, dest, HoprChannelsType.Balance.wrap(amount1));

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_wrapChannel(amount1, 0, 0, 1, HoprChannelsType.ChannelStatus.OPEN)))
        );
        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_expandChannel(hoprChannels.channelState(channelId))))
        );

        vm.clearMockedCalls();

        _helperOnlySafeMock(src, safeContract);
        _helperTokenTransferFromMock(safeContract, amount2);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelBalanceIncreased(
            channelId, _unwrapChannel(HoprChannelsType.Channel(
                HoprChannelsType.Balance.wrap(uint96(amount1 + amount2)),
                HoprChannelsType.TicketIndex.wrap(uint48(0)),
                HoprChannelsType.Timestamp.wrap(uint32(0)),
                HoprChannelsType.ChannelEpoch.wrap(uint24(1)),
                HoprChannelsType.ChannelStatus.OPEN
            ))
        );

        hoprChannels.fundChannelSafe(src, dest, HoprChannelsType.Balance.wrap(amount2));

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_wrapChannel(amount1 + amount2, 0, 0, 1, HoprChannelsType.ChannelStatus.OPEN)))
        );
        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_expandChannel(hoprChannels.channelState(channelId))))
        );

        vm.clearMockedCalls();
        vm.stopPrank();
    }

    function testRevert_fundChannelNoTokens(uint96 amount, address src, address dest, address safeContract) public {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest && safeContract != src && safeContract != dest);
        vm.assume(src != address(0) && dest != address(0) && safeContract != address(0));

        _helperNoSafeSetMock(src);
        _helperNoTokenTransferFromMock(src, amount);

        vm.expectRevert(HoprChannels.TokenTransferFailed.selector);

        vm.prank(src);
        hoprChannels.fundChannel(dest, HoprChannelsType.Balance.wrap(amount));

        vm.clearMockedCalls();

        // Test with Safe
        _helperOnlySafeMock(src, safeContract);
        _helperNoTokenTransferFromMock(safeContract, amount);

        vm.prank(safeContract);
        vm.expectRevert(HoprChannels.TokenTransferFailed.selector);

        hoprChannels.fundChannelSafe(src, dest, HoprChannelsType.Balance.wrap(amount));
    }

    function testRevert_fundChannelInvalidBalance(address src, address dest) public {
        vm.assume(src != dest);

        _helperNoSafeSetMock(src);

        vm.expectRevert(HoprChannels.InvalidBalance.selector);
        hoprChannels.fundChannel(dest, HoprChannelsType.Balance.wrap(0));

        vm.expectRevert(HoprChannels.BalanceExceedsGlobalPerChannelAllowance.selector);
        hoprChannels.fundChannel(dest, HoprChannelsType.Balance.wrap(uint96(MAX_USED_BALANCE) + 1));

        vm.clearMockedCalls();
    }

    function testRevert_fundChannelPendingToClose(address src, address dest, uint96 amount) public {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest);
        vm.assume(src != address(0));
        vm.assume(dest != address(0));

        _helperNoSafeSetMock(src);
        _helperTokenTransferFromMock(src, amount);
        hoprChannels._storeChannelStatus(src, dest, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE);

        vm.startPrank(src);

        vm.expectRevert(
            abi.encodeWithSelector(
                HoprChannels.WrongChannelState.selector, "cannot fund a channel that will close soon"
            )
        );
        hoprChannels.fundChannel(dest, HoprChannelsType.Balance.wrap(amount));

        hoprChannels._removeChannel(src, dest);
        vm.clearMockedCalls();
        vm.stopPrank();
    }

    function testRevert_fundChannelSameParty(address src, uint96 amount) public {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != address(0));

        vm.startPrank(src);

        vm.expectRevert(HoprChannels.SourceEqualsDestination.selector);
        hoprChannels.fundChannel(src, HoprChannelsType.Balance.wrap(amount));

        vm.expectRevert(abi.encodeWithSelector(HoprChannels.ZeroAddress.selector, "destination must not be empty"));
        hoprChannels.fundChannel(address(0), HoprChannelsType.Balance.wrap(amount));

        vm.stopPrank();
        vm.expectRevert(abi.encodeWithSelector(HoprChannels.ZeroAddress.selector, "source must not be empty"));
        vm.startPrank(address(0));

        hoprChannels.fundChannel(src, HoprChannelsType.Balance.wrap(amount));

        vm.clearMockedCalls();
        vm.stopPrank();
    }

    function test_closeIncomingChannel(
        address src,
        address dest,
        address safeContract,
        uint96 amount,
        uint48 ticketIndex
    )
        public
    {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest && safeContract != src && safeContract != dest);

        hoprChannels._storeChannel(dest, src, amount, ticketIndex, 0, 1, HoprChannelsType.ChannelStatus.OPEN);

        _helperNoSafeSetMock(src);
        _helperTokenTransferMock(dest, amount);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelClosed(hoprChannels._getChannelId(dest, src), _unwrapChannel(HoprChannelsType.Channel(
                HoprChannelsType.Balance.wrap(uint96(0)),
                HoprChannelsType.TicketIndex.wrap(uint48(0)),
                HoprChannelsType.Timestamp.wrap(uint32(0)),
                HoprChannelsType.ChannelEpoch.wrap(uint24(1)),
                HoprChannelsType.ChannelStatus.CLOSED
            ))
        );
        vm.prank(src);
        hoprChannels.closeIncomingChannel(dest);

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(dest, src))),
            keccak256(abi.encode(_wrapChannel(0, 0, 0, 1, HoprChannelsType.ChannelStatus.CLOSED)))
        );
        bytes32 channelId = keccak256(abi.encodePacked(dest, src));
        assertEq(
            keccak256(abi.encode(getChannelFromTuple(dest, src))),
            keccak256(abi.encode(_expandChannel(hoprChannels.channelState(channelId))))
        );

        vm.clearMockedCalls();

        _helperNoSafeSetMock(src);

        // Now let's assume there is a channel without funds
        hoprChannels._storeChannel(dest, src, 0, ticketIndex, 0, 2, HoprChannelsType.ChannelStatus.OPEN);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelClosed(hoprChannels._getChannelId(dest, src), _unwrapChannel(HoprChannelsType.Channel(
            HoprChannelsType.Balance.wrap(uint96(0)),
            HoprChannelsType.TicketIndex.wrap(uint48(0)),
            HoprChannelsType.Timestamp.wrap(uint32(0)),
            HoprChannelsType.ChannelEpoch.wrap(uint24(2)),
            HoprChannelsType.ChannelStatus.CLOSED
        )));

        vm.prank(src);
        hoprChannels.closeIncomingChannel(dest);

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(dest, src))),
            keccak256(abi.encode(_wrapChannel(0, 0, 0, 2, HoprChannelsType.ChannelStatus.CLOSED)))
        );
        assertEq(
            keccak256(abi.encode(getChannelFromTuple(dest, src))),
            keccak256(abi.encode(_expandChannel(hoprChannels.channelState(channelId))))
        );

        vm.clearMockedCalls();

        // clear state
        hoprChannels._removeChannel(src, dest);

        // Now test Safe contract
        _helperOnlySafeMock(src, safeContract);
        _helperTokenTransferMock(dest, amount);

        hoprChannels._storeChannel(dest, src, amount, ticketIndex, 0, 1, HoprChannelsType.ChannelStatus.OPEN);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelClosed(hoprChannels._getChannelId(dest, src), _unwrapChannel(HoprChannelsType.Channel(
                HoprChannelsType.Balance.wrap(uint96(0)),
                HoprChannelsType.TicketIndex.wrap(uint48(0)),
                HoprChannelsType.Timestamp.wrap(uint32(0)),
                HoprChannelsType.ChannelEpoch.wrap(uint24(1)),
                HoprChannelsType.ChannelStatus.CLOSED
            ))
        );

        vm.prank(safeContract);
        hoprChannels.closeIncomingChannelSafe(src, dest);

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(dest, src))),
            keccak256(abi.encode(_wrapChannel(0, 0, 0, 1, HoprChannelsType.ChannelStatus.CLOSED)))
        );
        assertEq(
            keccak256(abi.encode(getChannelFromTuple(dest, src))),
            keccak256(abi.encode(_expandChannel(hoprChannels.channelState(channelId))))
        );

        vm.clearMockedCalls();
    }

    function testRevert_closeIncomingChannelWrongChannelState(
        address src,
        address dest,
        uint96 amount,
        uint48 ticketIndex
    )
        public
    {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest);

        _helperNoSafeSetMock(src);

        hoprChannels._storeChannel(dest, src, amount, ticketIndex, 0, 1, HoprChannelsType.ChannelStatus.CLOSED);

        vm.expectRevert(
            abi.encodeWithSelector(
                HoprChannels.WrongChannelState.selector, "channel must have state OPEN or PENDING_TO_CLOSE"
            )
        );
        hoprChannels.closeIncomingChannel(dest);

        vm.clearMockedCalls();
    }

    function testRevert_closeIncomingChannelNoFunds(
        address src,
        address dest,
        uint96 amount,
        uint48 ticketIndex
    )
        public
    {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest);

        _helperNoSafeSetMock(src);
        _helperNoTokenTransferMock(dest, amount);

        hoprChannels._storeChannel(dest, src, amount, ticketIndex, 0, 1, HoprChannelsType.ChannelStatus.OPEN);

        vm.expectRevert(HoprChannels.TokenTransferFailed.selector);
        vm.prank(src);

        hoprChannels.closeIncomingChannel(dest);

        vm.clearMockedCalls();
    }

    function test_initiateOutgoingChannelClosure(
        address src,
        address dest,
        address safeContract,
        uint96 amount,
        uint48 ticketIndex,
        uint32 nextTimestamp
    )
        public
    {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest && safeContract != src && safeContract != dest);

        HoprChannelsType.Timestamp closureTime =
            HoprChannelsType.Timestamp.wrap(uint32(block.timestamp) + HoprChannelsType.Timestamp.unwrap(CLOSURE_NOTICE_PERIOD));
        nextTimestamp = uint32(
            bound(
                nextTimestamp,
                uint32(block.timestamp),
                uint32(type(uint32).max - HoprChannelsType.Timestamp.unwrap(CLOSURE_NOTICE_PERIOD))
            )
        );

        uint256 beginning = block.timestamp;

        _helperNoSafeSetMock(src);

        hoprChannels._storeChannel(src, dest, amount, ticketIndex, 0, 1, HoprChannelsType.ChannelStatus.OPEN);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit OutgoingChannelClosureInitiated(hoprChannels._getChannelId(src, dest), _unwrapChannel(HoprChannelsType.Channel(
                HoprChannelsType.Balance.wrap(uint96(amount)),
                HoprChannelsType.TicketIndex.wrap(uint48(ticketIndex)),
                closureTime,
                HoprChannelsType.ChannelEpoch.wrap(uint24(1)),
                HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
            ))
        );

        vm.prank(src);
        hoprChannels.initiateOutgoingChannelClosure(dest);

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(
                abi.encode(
                    _wrapChannel(
                        amount,
                        ticketIndex,
                        HoprChannelsType.Timestamp.unwrap(closureTime),
                        1,
                        HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
                    )
                )
            )
        );
        bytes32 channelId = keccak256(abi.encodePacked(src, dest));
        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_expandChannel(hoprChannels.channelState(channelId))))
        );

        // let's try to extend closureTime, safe as it's done by ticket issuer
        // which gives ticket redeemer more time to redeem tickets
        vm.warp(nextTimestamp);

        vm.prank(src);
        hoprChannels.initiateOutgoingChannelClosure(dest);

        HoprChannelsType.Timestamp nextClosureTime =
            HoprChannelsType.Timestamp.wrap(uint32(block.timestamp) + HoprChannelsType.Timestamp.unwrap(CLOSURE_NOTICE_PERIOD));

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(
                abi.encode(
                    _wrapChannel(
                        amount,
                        ticketIndex,
                        HoprChannelsType.Timestamp.unwrap(nextClosureTime),
                        1,
                        HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
                    )
                )
            )
        );

        vm.clearMockedCalls();

        // Clear state to test Safe functions

        vm.warp(beginning);

        _helperOnlySafeMock(src, safeContract);

        hoprChannels._storeChannel(src, dest, amount, ticketIndex, 0, 1, HoprChannelsType.ChannelStatus.OPEN);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit OutgoingChannelClosureInitiated(hoprChannels._getChannelId(src, dest), _unwrapChannel(HoprChannelsType.Channel(
                HoprChannelsType.Balance.wrap(uint96(amount)),
                HoprChannelsType.TicketIndex.wrap(uint48(ticketIndex)),
                closureTime,
                HoprChannelsType.ChannelEpoch.wrap(uint24(1)),
                HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
            )));

        vm.prank(safeContract);
        hoprChannels.initiateOutgoingChannelClosureSafe(src, dest);

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(
                abi.encode(
                    _wrapChannel(
                        amount,
                        ticketIndex,
                        HoprChannelsType.Timestamp.unwrap(closureTime),
                        1,
                        HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
                    )
                )
            )
        );
        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_expandChannel(hoprChannels.channelState(channelId))))
        );

        // let's try to extend closureTime, safe as it's done by ticket issuer
        // which gives ticket redeemer more time to redeem tickets
        vm.warp(nextTimestamp);

        vm.prank(safeContract);
        hoprChannels.initiateOutgoingChannelClosureSafe(src, dest);

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(
                abi.encode(
                    _wrapChannel(
                        amount,
                        ticketIndex,
                        HoprChannelsType.Timestamp.unwrap(nextClosureTime),
                        1,
                        HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
                    )
                )
            )
        );

        vm.clearMockedCalls();
    }

    function testRevert_initiateOutgoingChannelClosureWrongChannelState(
        address src,
        address dest,
        uint96 amount
    )
        public
    {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest);

        _helperNoSafeSetMock(src);

        hoprChannels._storeChannelStatus(src, dest, HoprChannelsType.ChannelStatus.CLOSED);

        vm.expectRevert(
            abi.encodeWithSelector(
                HoprChannels.WrongChannelState.selector, "channel must have state OPEN or PENDING_TO_CLOSE"
            )
        );
        vm.prank(src);
        hoprChannels.initiateOutgoingChannelClosure(dest);

        vm.clearMockedCalls();
    }

    function test_finalizeOutgoingChannelClosure(
        address src,
        address dest,
        address safeContract,
        uint96 amount,
        uint48 ticketIndex,
        uint32 closureTime,
        uint24 epoch
    )
        public
    {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        closureTime = uint32(
            bound(
                closureTime,
                block.timestamp,
                uint32(type(uint32).max) - HoprChannelsType.Timestamp.unwrap(CLOSURE_NOTICE_PERIOD) - 1
            )
        );
        vm.assume(src != address(0) && safeContract != address(0));
        vm.assume(src != dest && safeContract != src && safeContract != dest);

        _helperNoSafeSetMock(src);
        _helperTokenTransferMock(src, amount);

        hoprChannels._storeChannel(
            src, dest, amount, ticketIndex, closureTime, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
        );

        vm.warp(closureTime + HoprChannelsType.Timestamp.unwrap(CLOSURE_NOTICE_PERIOD) + 1);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelClosed(
            hoprChannels._getChannelId(src, dest),
            _unwrapChannel(HoprChannelsType.Channel(
                HoprChannelsType.Balance.wrap(uint96(0)),
                HoprChannelsType.TicketIndex.wrap(uint48(0)),
                HoprChannelsType.Timestamp.wrap(uint32(0)),
                HoprChannelsType.ChannelEpoch.wrap(epoch),
                HoprChannelsType.ChannelStatus.CLOSED
            ))
        );

        vm.prank(src);
        hoprChannels.finalizeOutgoingChannelClosure(dest);

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_wrapChannel(0, 0, 0, epoch, HoprChannelsType.ChannelStatus.CLOSED)))
        );

        vm.clearMockedCalls();

        // Now let's test safe integration

        _helperOnlySafeMock(src, safeContract);
        _helperTokenTransferMock(safeContract, amount);

        hoprChannels._storeChannel(
            src, dest, amount, ticketIndex, closureTime, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
        );

        vm.warp(closureTime + HoprChannelsType.Timestamp.unwrap(CLOSURE_NOTICE_PERIOD) + 1);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelClosed(
            hoprChannels._getChannelId(src, dest),
            _unwrapChannel(HoprChannelsType.Channel(
                HoprChannelsType.Balance.wrap(uint96(0)),
                HoprChannelsType.TicketIndex.wrap(uint48(0)),
                HoprChannelsType.Timestamp.wrap(uint32(0)),
                HoprChannelsType.ChannelEpoch.wrap(epoch),
                HoprChannelsType.ChannelStatus.CLOSED
            ))
        );

        vm.prank(safeContract);
        hoprChannels.finalizeOutgoingChannelClosureSafe(src, dest);

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_wrapChannel(0, 0, 0, epoch, HoprChannelsType.ChannelStatus.CLOSED)))
        );

        vm.clearMockedCalls();
    }

    function testRevert_finalizeOutgoingChannelClosureWrongChannelState(address src, address dest) public {
        vm.assume(src != dest);

        _helperNoSafeSetMock(src);

        hoprChannels._storeChannelStatus(src, dest, HoprChannelsType.ChannelStatus.CLOSED);

        vm.startPrank(src);
        vm.expectRevert(
            abi.encodeWithSelector(HoprChannels.WrongChannelState.selector, "channel state must be PENDING_TO_CLOSE")
        );
        hoprChannels.finalizeOutgoingChannelClosure(dest);

        hoprChannels._storeChannelStatus(src, dest, HoprChannelsType.ChannelStatus.OPEN);

        vm.expectRevert(
            abi.encodeWithSelector(HoprChannels.WrongChannelState.selector, "channel state must be PENDING_TO_CLOSE")
        );
        hoprChannels.finalizeOutgoingChannelClosure(dest);

        vm.clearMockedCalls();
        vm.stopPrank();
    }

    function testRevert_finalizeOutgoingChannelClosureNotDue(
        address src,
        address dest,
        uint96 amount,
        uint48 ticketIndex,
        uint32 closureTime,
        uint24 epoch
    )
        public
    {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        closureTime = uint32(
            bound(
                closureTime,
                block.timestamp + 1,
                uint32(type(uint32).max) - HoprChannelsType.Timestamp.unwrap(CLOSURE_NOTICE_PERIOD) - 1
            )
        );
        vm.assume(src != dest);

        _helperNoSafeSetMock(src);

        hoprChannels._storeChannel(
            src, dest, amount, ticketIndex, closureTime, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
        );

        vm.startPrank(src);
        vm.expectRevert(HoprChannels.NoticePeriodNotDue.selector);
        hoprChannels.finalizeOutgoingChannelClosure(dest);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function testRevert_finalizeOutgoingChannelClosureTokenTransfer(
        address src,
        address dest,
        uint96 amount,
        uint48 ticketIndex,
        uint32 closureTime,
        uint24 epoch
    )
        public
    {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        closureTime = uint32(
            bound(
                closureTime,
                block.timestamp + 1,
                uint32(type(uint32).max) - HoprChannelsType.Timestamp.unwrap(CLOSURE_NOTICE_PERIOD) - 1
            )
        );
        vm.assume(src != dest);

        _helperNoSafeSetMock(src);
        _helperNoTokenTransferMock(src, amount);

        vm.warp(closureTime + 1);

        hoprChannels._storeChannel(
            src, dest, amount, ticketIndex, closureTime, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
        );

        vm.startPrank(src);
        vm.expectRevert(HoprChannels.TokenTransferFailed.selector);
        hoprChannels.finalizeOutgoingChannelClosure(dest);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function testFuzz_redeemTicket(
        uint256 privKeyA,
        uint256 privKeyB,
        address safeContract,
        uint256 porSecret,
        uint96 channelAmount,
        uint96 amount,
        uint48 maxTicketIndex,
        uint24 epoch,
        uint48 channelTicketIndex
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        vm.assume(privKeyA != privKeyB);
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        channelAmount = uint96(bound(channelAmount, amount, MAX_USED_BALANCE));
        channelTicketIndex = uint48(bound(channelTicketIndex, 0, type(uint48).max - BASE_INDEX_OFFSET - 1));
        maxTicketIndex = uint48(bound(maxTicketIndex, channelTicketIndex + 1, type(uint48).max - BASE_INDEX_OFFSET));

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);
        vm.assume(safeContract != address(0) && safeContract != src && safeContract != dest);

        _helperNoSafeSetMock(dest);
        _helperTokenTransferMock(dest, amount);

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            epoch,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_100),
            porSecret
        );

        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.OPEN
        );

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);

        HoprChannelsType.Channel memory tmpChannel = getChannelFromTuple(src, dest);
        tmpChannel.balance = HoprChannelsType.Balance.wrap(channelAmount - amount);
        tmpChannel.ticketIndex = HoprChannelsType.TicketIndex.wrap(maxTicketIndex + BASE_INDEX_OFFSET);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelBalanceDecreased(redeemable.data.channelId, _unwrapChannel(tmpChannel));

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit TicketRedeemed(redeemable.data.channelId, _unwrapChannel(tmpChannel));
        vm.prank(dest);
        hoprChannels.redeemTicket(redeemable, vrf);

        // Now let's assume the channel is PENDING_TO_CLOSE
        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
        );

        tmpChannel = getChannelFromTuple(src, dest);
        tmpChannel.balance = HoprChannelsType.Balance.wrap(channelAmount - amount);
        tmpChannel.ticketIndex = HoprChannelsType.TicketIndex.wrap(maxTicketIndex + BASE_INDEX_OFFSET);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelBalanceDecreased(redeemable.data.channelId, _unwrapChannel(tmpChannel));

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit TicketRedeemed(redeemable.data.channelId,_unwrapChannel(tmpChannel));

        vm.prank(dest);
        hoprChannels.redeemTicket(redeemable, vrf);

        // Reset to test safe integration
        vm.clearMockedCalls();
        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.OPEN
        );

        _helperOnlySafeMock(dest, safeContract);
        _helperTokenTransferMock(safeContract, amount);

        tmpChannel = getChannelFromTuple(src, dest);
        tmpChannel.balance = HoprChannelsType.Balance.wrap(channelAmount - amount);
        tmpChannel.ticketIndex = HoprChannelsType.TicketIndex.wrap(maxTicketIndex + BASE_INDEX_OFFSET);

        // Now test Safe integration
        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelBalanceDecreased(redeemable.data.channelId, _unwrapChannel(tmpChannel));

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit TicketRedeemed(redeemable.data.channelId, _unwrapChannel(tmpChannel));
        vm.prank(safeContract);
        hoprChannels.redeemTicketSafe(dest, redeemable, vrf);

        // Now let's assume the channel is PENDING_TO_CLOSE
        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
        );
        tmpChannel = getChannelFromTuple(src, dest);
        tmpChannel.balance = HoprChannelsType.Balance.wrap(channelAmount - amount);
        tmpChannel.ticketIndex = HoprChannelsType.TicketIndex.wrap(maxTicketIndex + BASE_INDEX_OFFSET);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelBalanceDecreased(redeemable.data.channelId, _unwrapChannel(tmpChannel));

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit TicketRedeemed(redeemable.data.channelId, _unwrapChannel(tmpChannel));

        vm.prank(safeContract);
        hoprChannels.redeemTicketSafe(dest, redeemable, vrf);
    }

    function testRevert_CannotRedeemSameWinningTicketMultipleTimes(
        uint256 privKeyA,
        uint256 privKeyB,
        uint256 porSecret,
        uint96 amount,
        uint96 channelAmount,
        uint24 epoch
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        vm.assume(privKeyA != privKeyB);

        channelAmount = uint96(bound(channelAmount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        amount = uint96(bound(amount, MIN_USED_BALANCE, channelAmount));
        uint48 maxTicketIndex = uint48(6);
        uint48 channelTicketIndex = uint48(1);

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);

        _helperNoSafeSetMock(dest);
        _helperTokenTransferMock(dest, amount);

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            epoch,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_100),
            porSecret
        );

        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.OPEN
        );

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);
        
        HoprChannelsType.Channel memory tmpChannel = getChannelFromTuple(src, dest);
        tmpChannel.balance = HoprChannelsType.Balance.wrap(channelAmount - amount);
        tmpChannel.ticketIndex = HoprChannelsType.TicketIndex.wrap(maxTicketIndex + BASE_INDEX_OFFSET);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit TicketRedeemed(redeemable.data.channelId, _unwrapChannel(tmpChannel));
        vm.prank(dest);
        hoprChannels.redeemTicket(redeemable, vrf);

        for (uint256 i = 1; i < uint256(maxTicketIndex - channelTicketIndex); i++) {
            vm.expectRevert(HoprChannels.InvalidTicketIndex.selector);
            vm.prank(dest);
            hoprChannels.redeemTicket(redeemable, vrf);
        }
    }

    /// forge-lint:disable-next-item(mixed-case-variable)
    function test_redeemTicket_bidirectional(
        uint256 privKeyA,
        uint256 privKeyB,
        uint256 porSecret,
        uint96 channelABAmount,
        uint96 channelBAAmount,
        uint96 amount,
        uint48 maxTicketIndex,
        uint24 epoch,
        uint48 channelTicketIndex
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        vm.assume(privKeyA != privKeyB);
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        channelABAmount = uint96(bound(channelABAmount, amount, MAX_USED_BALANCE));
        channelBAAmount = uint96(bound(channelBAAmount, 0, type(uint96).max - amount));
        channelTicketIndex = uint48(bound(channelTicketIndex, 0, type(uint48).max - BASE_INDEX_OFFSET - 1));
        maxTicketIndex = uint48(bound(maxTicketIndex, channelTicketIndex + 1, type(uint48).max - BASE_INDEX_OFFSET));

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);

        _helperNoSafeSetMock(dest);
        _helperTokenTransferMock(dest, amount);

        hoprChannels._storeChannel(
            src, dest, channelABAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
        );
        hoprChannels._storeChannel(
            dest, src, channelBAAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.PENDING_TO_CLOSE
        );

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            epoch,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_100),
            porSecret
        );

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);

        HoprChannelsType.Channel memory abChannel = getChannelFromTuple(src, dest);
        abChannel.balance = HoprChannelsType.Balance.wrap(channelABAmount - amount);
        abChannel.ticketIndex = HoprChannelsType.TicketIndex.wrap(maxTicketIndex + BASE_INDEX_OFFSET);
        HoprChannelsType.Channel memory baChannel = getChannelFromTuple(dest, src);
        baChannel.balance = HoprChannelsType.Balance.wrap(channelBAAmount + amount);

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelBalanceDecreased(redeemable.data.channelId, _unwrapChannel(abChannel));

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit TicketRedeemed(redeemable.data.channelId, _unwrapChannel(abChannel));

        vm.expectEmit(true, false, false, true, address(hoprChannels));
        emit ChannelBalanceIncreased(
            hoprChannels._getChannelId(dest, src), _unwrapChannel(baChannel)
        );

        vm.prank(dest);
        hoprChannels.redeemTicket(redeemable, vrf);

        vm.clearMockedCalls();
    }

    function testRevert_redeemTicketZeroWinProb(
        uint256 privKeyA,
        uint256 privKeyB,
        uint256 porSecret,
        uint96 channelAmount,
        uint96 amount,
        uint48 maxTicketIndex,
        uint24 epoch,
        uint48 channelTicketIndex
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        vm.assume(privKeyA != privKeyB);
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        channelAmount = uint96(bound(channelAmount, amount, MAX_USED_BALANCE));
        channelTicketIndex = uint48(bound(channelTicketIndex, 0, type(uint48).max - BASE_INDEX_OFFSET - 1));
        maxTicketIndex = uint48(bound(maxTicketIndex, channelTicketIndex + 1, type(uint48).max - BASE_INDEX_OFFSET));

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);

        _helperNoSafeSetMock(dest);
        _helperTokenTransferMock(dest, amount);

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            epoch,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_0),
            porSecret
        );

        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.OPEN
        );

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);

        vm.prank(dest);

        vm.expectRevert(HoprChannels.TicketIsNotAWin.selector);
        hoprChannels.redeemTicket(redeemable, vrf);
        vm.clearMockedCalls();
    }

    function testRevert_redeemTicketInsufficientChannelFunds(
        uint256 privKeyA,
        uint256 privKeyB,
        uint256 porSecret,
        uint96 channelAmount,
        uint96 amount,
        uint48 maxTicketIndex,
        uint24 epoch,
        uint48 channelTicketIndex
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        vm.assume(privKeyA != privKeyB);
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        channelTicketIndex = uint48(bound(channelTicketIndex, 0, type(uint48).max - BASE_INDEX_OFFSET - 1));
        maxTicketIndex = uint48(bound(maxTicketIndex, channelTicketIndex + 1, type(uint48).max - BASE_INDEX_OFFSET));
        vm.assume(channelAmount < amount);

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);

        _helperNoSafeSetMock(dest);
        _helperTokenTransferMock(dest, amount);

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            epoch,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_0),
            porSecret
        );

        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.OPEN
        );

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);

        vm.prank(dest);

        vm.expectRevert(HoprChannels.InsufficientChannelBalance.selector);
        hoprChannels.redeemTicket(redeemable, vrf);
        vm.clearMockedCalls();
    }

    function testRevert_redeemTicketInsufficientAccountFunds(
        uint256 privKeyA,
        uint256 privKeyB,
        uint256 porSecret,
        uint96 channelAmount,
        uint96 amount,
        uint48 maxTicketIndex,
        uint24 epoch,
        uint48 channelTicketIndex
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        channelAmount = uint96(bound(channelAmount, amount, MAX_USED_BALANCE));
        channelTicketIndex = uint48(bound(channelTicketIndex, 0, type(uint48).max - BASE_INDEX_OFFSET - 1));
        maxTicketIndex = uint48(bound(maxTicketIndex, channelTicketIndex + 1, type(uint48).max - BASE_INDEX_OFFSET));
        vm.assume(privKeyA != privKeyB);

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);

        _helperNoSafeSetMock(dest);
        _helperNoTokenTransferMock(dest, amount);

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            epoch,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_100),
            porSecret
        );

        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.OPEN
        );

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);

        vm.prank(dest);

        vm.expectRevert(HoprChannels.TokenTransferFailed.selector);
        hoprChannels.redeemTicket(redeemable, vrf);
        vm.clearMockedCalls();
    }

    function testRevert_redeemTicketWrongChannelState(
        uint256 privKeyA,
        uint256 privKeyB,
        uint256 porSecret,
        uint96 channelAmount,
        uint96 amount,
        uint48 maxTicketIndex,
        uint24 epoch,
        uint48 channelTicketIndex
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        vm.assume(privKeyA != privKeyB);
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        channelAmount = uint96(bound(channelAmount, amount, MAX_USED_BALANCE));
        channelTicketIndex = uint48(bound(channelTicketIndex, 0, type(uint48).max - BASE_INDEX_OFFSET - 1));
        maxTicketIndex = uint48(bound(maxTicketIndex, channelTicketIndex + 1, type(uint48).max - BASE_INDEX_OFFSET));

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);

        _helperNoSafeSetMock(dest);
        _helperTokenTransferMock(dest, amount);

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            epoch,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_0),
            porSecret
        );

        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.CLOSED
        );

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);

        vm.prank(dest);

        vm.expectRevert(
            abi.encodeWithSelector(
                HoprChannels.WrongChannelState.selector, "spending channel must be OPEN or PENDING_TO_CLOSE"
            )
        );
        hoprChannels.redeemTicket(redeemable, vrf);
        vm.clearMockedCalls();
    }

    function testRevert_redeemTicketWrongEpoch(
        uint256 privKeyA,
        uint256 privKeyB,
        uint256 porSecret,
        uint96 channelAmount,
        uint96 amount,
        uint48 maxTicketIndex,
        uint48 channelTicketIndex
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        vm.assume(privKeyA != privKeyB);
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        channelAmount = uint96(bound(channelAmount, amount, MAX_USED_BALANCE));
        channelTicketIndex = uint48(bound(channelTicketIndex, 0, type(uint48).max - BASE_INDEX_OFFSET - 1));
        maxTicketIndex = uint48(bound(maxTicketIndex, channelTicketIndex + 1, type(uint48).max - BASE_INDEX_OFFSET));

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);

        _helperNoSafeSetMock(dest);
        _helperTokenTransferMock(dest, amount);

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            1,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_0),
            porSecret
        );

        hoprChannels._storeChannel(src, dest, channelAmount, channelTicketIndex, 0, 2, HoprChannelsType.ChannelStatus.OPEN);

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);

        vm.prank(dest);

        vm.expectRevert(abi.encodeWithSelector(HoprChannels.WrongChannelState.selector, "channel epoch must match"));
        hoprChannels.redeemTicket(redeemable, vrf);
        vm.clearMockedCalls();
    }

    function testRevert_redeemTicketInvalidVRFProof(
        uint256 privKeyA,
        uint256 privKeyB,
        uint256 porSecret,
        uint96 channelAmount,
        uint96 amount,
        uint48 maxTicketIndex,
        uint24 epoch,
        uint48 channelTicketIndex
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        vm.assume(privKeyA != privKeyB);
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        channelAmount = uint96(bound(channelAmount, amount, MAX_USED_BALANCE));
        channelTicketIndex = uint48(bound(channelTicketIndex, 0, type(uint48).max - BASE_INDEX_OFFSET - 1));
        maxTicketIndex = uint48(bound(maxTicketIndex, channelTicketIndex + 1, type(uint48).max - BASE_INDEX_OFFSET));

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);

        _helperNoSafeSetMock(dest);
        _helperTokenTransferMock(dest, amount);

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            epoch,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_100),
            porSecret
        );

        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.OPEN
        );

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);

        vrf.h = 1;
        vrf.hVx = vrf.vx;
        vrf.hVy = vrf.vy;
        vm.prank(dest);

        vm.expectRevert(HoprChannels.InvalidVRFProof.selector);
        hoprChannels.redeemTicket(redeemable, vrf);

        vm.clearMockedCalls();
    }

    function testRevert_redeemTicketInvalidSignature(
        uint256 privKeyA,
        uint256 privKeyB,
        uint256 porSecret,
        uint96 channelAmount,
        uint96 amount,
        uint48 maxTicketIndex,
        uint24 epoch,
        uint48 channelTicketIndex
    )
        public
    {
        porSecret = bound(porSecret, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyA = bound(privKeyA, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        privKeyB = bound(privKeyB, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);
        vm.assume(privKeyA != privKeyB);
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        channelAmount = uint96(bound(channelAmount, amount, MAX_USED_BALANCE));
        channelTicketIndex = uint48(bound(channelTicketIndex, 0, type(uint48).max - BASE_INDEX_OFFSET - 1));
        maxTicketIndex = uint48(bound(maxTicketIndex, channelTicketIndex + 1, type(uint48).max - BASE_INDEX_OFFSET));

        address src = vm.addr(privKeyA);
        address dest = vm.addr(privKeyB);

        _helperNoSafeSetMock(dest);
        _helperTokenTransferMock(dest, amount);

        RedeemTicketArgBuilder memory args = RedeemTicketArgBuilder(
            privKeyA,
            privKeyB,
            hoprChannels.domainSeparator(),
            src,
            dest,
            amount,
            maxTicketIndex,
            epoch,
            HoprChannelsType.WinProb.unwrap(WIN_PROB_100),
            porSecret
        );

        hoprChannels._storeChannel(
            src, dest, channelAmount, channelTicketIndex, 0, epoch, HoprChannelsType.ChannelStatus.OPEN
        );

        (HoprChannels.RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory vrf) =
            CryptoUtils.getRedeemableTicket(args);

        (uint8 v, bytes32 r, bytes32 s) =
            CryptoUtils.decompressSignature(redeemable.signature.r, redeemable.signature.vs);
        if (v == 27) {
            // v == 27 or v == 28
            v = 28;
        } else {
            v = 27;
        }
        HoprCrypto.CompactSignature memory tweakedSig = toCompactSignature(v, r, s);
        redeemable.signature.vs = tweakedSig.vs;

        vm.prank(dest);

        vm.expectRevert(HoprChannels.InvalidTicketSignature.selector);
        hoprChannels.redeemTicket(redeemable, vrf);

        vm.clearMockedCalls();
    }


    function test_tokensReceivedSingle(
        address operator,
        address safeContract,
        address src,
        address dest,
        uint96 amountA,
        uint96 amountB,
        bytes memory operatorData
    )
        public
    {
        amountA = uint96(bound(amountA, MIN_USED_BALANCE, MAX_USED_BALANCE));
        amountB = uint96(bound(amountB, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest && safeContract != src && safeContract != dest);
        vm.assume(src != address(0) && dest != address(0) && safeContract != address(0));
        vm.assume(amountA != amountB);

        _helperNoSafeSetMock(src);

        vm.prank(address(hoprToken));
        hoprChannels.tokensReceived(
            operator, src, address(hoprChannels), amountA, abi.encodePacked(src, dest), operatorData
        );

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_wrapChannel(amountA, 0, 0, 1, HoprChannelsType.ChannelStatus.OPEN)))
        );

        // from != src (called by Safe)
        vm.clearMockedCalls();
        _helperOnlySafeMock(src, safeContract);
        hoprChannels._removeChannel(src, dest);

        vm.prank(address(hoprToken));
        hoprChannels.tokensReceived(
            operator, safeContract, address(hoprChannels), amountA, abi.encodePacked(src, dest), operatorData
        );

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_wrapChannel(amountA, 0, 0, 1, HoprChannelsType.ChannelStatus.OPEN)))
        );
    }

    function test_tokensReceivedMulti(
        address operator,
        address safeContract,
        address src,
        address dest,
        uint96 amountA,
        uint96 amountB,
        bytes memory operatorData
    )
        public
    {
        amountA = uint96(bound(amountA, MIN_USED_BALANCE, MAX_USED_BALANCE));
        amountB = uint96(bound(amountB, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest && safeContract != src && safeContract != dest);
        vm.assume(src != address(0) && dest != address(0) && safeContract != address(0));
        vm.assume(amountA != amountB);

        vm.prank(address(hoprToken));
        hoprChannels.tokensReceived(
            operator,
            src,
            address(hoprChannels),
            amountA + amountB,
            abi.encodePacked(src, amountA, dest, amountB),
            operatorData
        );

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(src, dest))),
            keccak256(abi.encode(_wrapChannel(amountA, 0, 0, 1, HoprChannelsType.ChannelStatus.OPEN)))
        );

        assertEq(
            keccak256(abi.encode(getChannelFromTuple(dest, src))),
            keccak256(abi.encode(_wrapChannel(amountB, 0, 0, 1, HoprChannelsType.ChannelStatus.OPEN)))
        );

        vm.clearMockedCalls();
    }

    function testRevert_tokensReceivedWrongToken(
        address caller,
        address someContract,
        uint96 correctAmount,
        address operator,
        bytes memory operatorData,
        address from,
        address to,
        uint256 amount,
        address src,
        address dest
    )
        public
    {
        amount = bound(amount, uint256(type(uint96).max) + 1, type(uint256).max);
        correctAmount = uint96(bound(correctAmount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != dest);
        vm.assume(src != address(0) && dest != address(0));
        vm.assume(someContract != address(hoprChannels));

        _helperNoSafeSetMock(src);
        vm.expectRevert(HoprChannels.BalanceExceedsGlobalPerChannelAllowance.selector);
        vm.prank(address(hoprToken));
        hoprChannels.tokensReceived(
            operator, src, address(hoprChannels), amount, abi.encodePacked(src, dest), operatorData
        );

        vm.expectRevert(HoprChannels.InvalidBalance.selector);
        vm.prank(address(hoprToken));
        hoprChannels.tokensReceived(
            operator, src, address(hoprChannels), uint256(0), abi.encodePacked(src, dest), operatorData
        );
        vm.clearMockedCalls();

        vm.assume(caller != address(hoprToken));
        vm.expectRevert(HoprChannels.WrongToken.selector);
        vm.prank(caller);
        hoprChannels.tokensReceived(operator, from, to, correctAmount, abi.encodePacked(src, dest), operatorData);

        vm.expectRevert(HoprChannels.InvalidTokenRecipient.selector);
        vm.prank(address(hoprToken));
        hoprChannels.tokensReceived(
            operator, from, someContract, correctAmount, abi.encodePacked(src, dest), operatorData
        );
    }

    function testRevert_tokensReceivedWrongABI(
        address operator,
        bytes memory operatorData,
        address from,
        uint256 amount,
        bytes memory userData
    )
        public
    {
        amount = bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE);

        vm.assume(userData.length != 0 && userData.length != 40 && userData.length != 64);

        vm.expectRevert(HoprChannels.InvalidTokensReceivedUsage.selector);
        vm.prank(address(hoprToken));
        hoprChannels.tokensReceived(operator, from, address(hoprChannels), amount, userData, operatorData);
    }

    function testRevert_tokensReceivedFundMulti(
        uint256 someAmount,
        address operator,
        bytes memory operatorData,
        address from,
        uint96 amountA,
        uint96 amountB,
        address src,
        address dest
    )
        public
    {
        amountA = uint96(bound(amountA, MIN_USED_BALANCE, MAX_USED_BALANCE));
        amountB = uint96(bound(amountB, MIN_USED_BALANCE, MAX_USED_BALANCE));

        vm.assume(someAmount < type(uint96).max && someAmount != amountA + amountB);

        vm.assume(src != dest);
        vm.assume(src != address(0) && dest != address(0));

        vm.prank(address(hoprToken));
        vm.expectRevert(HoprChannels.InvalidBalance.selector);
        hoprChannels.tokensReceived(
            operator,
            from,
            address(hoprChannels),
            someAmount,
            abi.encodePacked(src, amountA, dest, amountB),
            operatorData
        );
    }

    function testRevert_tokensReceivedInvalidBalance(
        address safeContract,
        address src,
        address dest,
        uint256 amountTooSmall,
        uint256 amountTooLarge,
        address operator
    )
        public
    {
        vm.assume(src != dest && src != safeContract && dest != safeContract);
        vm.assume(src != address(0) && dest != address(0));
        vm.assume(safeContract != address(0));
        vm.assume(operator != address(0));

        vm.assume(amountTooSmall < uint256(MIN_USED_BALANCE) - 1);
        amountTooLarge = bound(amountTooLarge, uint256(MAX_USED_BALANCE) + 1, type(uint96).max);
        HoprChannelsType.Balance balanceTooSmall = HoprChannelsType.Balance.wrap(uint96(amountTooSmall));
        HoprChannelsType.Balance balanceTooLarge = HoprChannelsType.Balance.wrap(uint96(amountTooLarge));

        vm.startPrank(address(hoprToken));

        // a. from == src (called by node directly)
        // userData.length == ERC777_HOOK_FUND_CHANNEL_SIZE
        _helperNoSafeSetMock(src);

        vm.expectRevert(HoprChannels.InvalidBalance.selector);
        hoprChannels.tokensReceived(
            operator, src, address(hoprChannels), amountTooSmall, abi.encodePacked(src, dest), hex""
        );
        vm.expectRevert(HoprChannels.BalanceExceedsGlobalPerChannelAllowance.selector);
        hoprChannels.tokensReceived(
            operator, src, address(hoprChannels), amountTooLarge, abi.encodePacked(src, dest), hex""
        );

        // userData.length == ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE
        vm.expectRevert(HoprChannels.InvalidBalance.selector);
        hoprChannels.tokensReceived(
            operator,
            src,
            address(hoprChannels),
            amountTooSmall * 2,
            abi.encodePacked(src, balanceTooSmall, dest, balanceTooSmall),
            hex""
        );
        vm.expectRevert(HoprChannels.BalanceExceedsGlobalPerChannelAllowance.selector);
        hoprChannels.tokensReceived(
            operator,
            src,
            address(hoprChannels),
            amountTooLarge * 2,
            abi.encodePacked(src, balanceTooLarge, dest, balanceTooLarge),
            hex""
        );

        // b. from != src (called by Safe)
        vm.clearMockedCalls();
        _helperOnlySafeMock(src, safeContract);
        hoprChannels._removeChannel(src, dest);

        // userData.length == ERC777_HOOK_FUND_CHANNEL_SIZE
        vm.expectRevert(HoprChannels.InvalidBalance.selector);
        hoprChannels.tokensReceived(
            operator, safeContract, address(hoprChannels), amountTooSmall, abi.encodePacked(safeContract, dest), hex""
        );
        vm.expectRevert(HoprChannels.BalanceExceedsGlobalPerChannelAllowance.selector);
        hoprChannels.tokensReceived(
            operator, safeContract, address(hoprChannels), amountTooLarge, abi.encodePacked(safeContract, dest), hex""
        );

        // userData.length == ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE
        vm.expectRevert(HoprChannels.BalanceExceedsGlobalPerChannelAllowance.selector);
        hoprChannels.tokensReceived(
            operator,
            safeContract,
            address(hoprChannels),
            amountTooLarge * 2,
            abi.encodePacked(src, balanceTooLarge, dest, balanceTooLarge),
            hex""
        );
        vm.clearMockedCalls();
        vm.stopPrank();
    }

    function testRevert_tokensReceivedSameParty(address safeContract, address src, uint96 amount) public {
        amount = uint96(bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE));
        vm.assume(src != address(0));
        vm.assume(safeContract != src);

        address operator = address(0);
        HoprChannelsType.Balance balance = HoprChannelsType.Balance.wrap(amount);

        vm.startPrank(address(hoprToken));

        // a. from == src (called by node directly)
        // userData.length == ERC777_HOOK_FUND_CHANNEL_SIZE
        _helperNoSafeSetMock(src);

        vm.expectRevert(HoprChannels.SourceEqualsDestination.selector);
        hoprChannels.tokensReceived(operator, src, address(hoprChannels), amount, abi.encodePacked(src, src), hex"");

        // userData.length == ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE
        vm.expectRevert(HoprChannels.SourceEqualsDestination.selector);
        hoprChannels.tokensReceived(
            operator, src, address(hoprChannels), amount * 2, abi.encodePacked(src, balance, src, balance), hex""
        );

        // b. from != src (called by Safe)
        vm.clearMockedCalls();
        _helperOnlySafeMock(src, safeContract);
        // hoprChannels._removeChannel(src, dest);

        // userData.length == ERC777_HOOK_FUND_CHANNEL_SIZE
        vm.expectRevert(HoprChannels.SourceEqualsDestination.selector);
        hoprChannels.tokensReceived(
            operator, safeContract, address(hoprChannels), amount, abi.encodePacked(src, src), hex""
        );

        // userData.length == ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE
        vm.expectRevert(HoprChannels.SourceEqualsDestination.selector);
        hoprChannels.tokensReceived(
            operator,
            safeContract,
            address(hoprChannels),
            amount * 2,
            abi.encodePacked(src, balance, src, balance),
            hex""
        );
        vm.clearMockedCalls();
        vm.stopPrank();
    }

    function testRevert_tokensReceivedSafeIntegration(
        address someAccount,
        address operator,
        bytes memory operatorData,
        address safeContract,
        uint256 amount,
        address src,
        address dest
    )
        public
    {
        amount = bound(amount, MIN_USED_BALANCE, MAX_USED_BALANCE);

        vm.assume(src != dest && safeContract != src && safeContract != dest);
        vm.assume(src != address(0) && dest != address(0) && safeContract != address(0) && someAccount != address(0));
        vm.assume(someAccount != src);

        _helperOnlySafeMock(src, safeContract);
        vm.expectRevert(HoprMultiSig.ContractNotResponsible.selector);
        vm.prank(address(hoprToken));
        hoprChannels.tokensReceived(
            operator, src, address(hoprChannels), amount, abi.encodePacked(src, dest), operatorData
        );

        vm.clearMockedCalls();
        _helperNoSafeSetMock(src);
        vm.expectRevert(HoprMultiSig.ContractNotResponsible.selector);
        vm.prank(address(hoprToken));
        hoprChannels.tokensReceived(
            operator, someAccount, address(hoprChannels), amount, abi.encodePacked(src, dest), operatorData
        );

        vm.clearMockedCalls();
    }

    function testFuzz_DomainSeparator(uint64 newId) public {
        // chain ID must be less than 2^64 - 1
        uint256 newChainId = bound(uint256(newId), 1, 0xFFFFFFFFFFFFFFFE);
        uint256 oldChainId = block.chainid;
        vm.assume(newChainId != oldChainId);
        bytes32 domainSeparatorOnDeployment = hoprChannels.domainSeparator();

        // call updateDomainSeparator when chainid is the same
        hoprChannels.updateDomainSeparator();
        assertEq(hoprChannels.domainSeparator(), domainSeparatorOnDeployment);

        // call updateDomainSeparator when chainid is different
        vm.chainId(newChainId);
        vm.expectEmit(true, true, false, false, address(hoprChannels));
        emit DomainSeparatorUpdated(
            keccak256(
                abi.encode(
                    keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                    keccak256(bytes("HoprChannels")),
                    keccak256(bytes(hoprChannels.VERSION())),
                    newChainId,
                    address(hoprChannels)
                )
            )
        );
        hoprChannels.updateDomainSeparator();
        assertTrue(hoprChannels.domainSeparator() != domainSeparatorOnDeployment);
        vm.chainId(oldChainId);
    }

    function testFuzz_LedgerDomainSeparator(uint64 newId) public {
        // chain ID must be less than 2^64 - 1
        uint256 newChainId = bound(uint256(newId), 1, 0xFFFFFFFFFFFFFFFE);
        uint256 oldChainId = block.chainid;
        vm.assume(newChainId != oldChainId);
        bytes32 ledgerDomainSeparatorOnDeployment = hoprChannels.ledgerDomainSeparator();

        // call updateLedgerDomainSeparator when chainid is the same
        hoprChannels.updateLedgerDomainSeparator();
        assertEq(hoprChannels.ledgerDomainSeparator(), ledgerDomainSeparatorOnDeployment);

        // call updateLedgerDomainSeparator when chainid is different
        vm.chainId(newChainId);
        vm.expectEmit(true, true, false, false, address(hoprChannels));
        emit LedgerDomainSeparatorUpdated(
            keccak256(
                abi.encode(
                    keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                    keccak256(bytes("HoprLedger")),
                    keccak256(bytes(hoprChannels.LEDGER_VERSION())),
                    newChainId,
                    address(hoprChannels)
                )
            )
        );
        hoprChannels.updateLedgerDomainSeparator();
        assertTrue(hoprChannels.ledgerDomainSeparator() != ledgerDomainSeparatorOnDeployment);
        vm.chainId(oldChainId);
    }

    /**
     * @dev mock a return of safe registered to node
     */
    function _helperNoSafeSetMock(address node) private {
        vm.mockCall(
            address(hoprNodeSafeRegistry),
            abi.encodeWithSelector(hoprNodeSafeRegistry.nodeToSafe.selector, node),
            abi.encode(address(0))
        );
    }

    /**
     * @dev mock a return of safe registsered to node
     */
    function _helperOnlySafeMock(address node, address caller) private {
        vm.mockCall(
            address(hoprNodeSafeRegistry),
            abi.encodeWithSelector(hoprNodeSafeRegistry.nodeToSafe.selector, node),
            abi.encode(caller)
        );
    }

    function _helperTokenTransferFromMock(address owner, uint256 amount) private {
        vm.mockCall(
            address(hoprToken),
            abi.encodeWithSelector(hoprToken.transferFrom.selector, owner, address(hoprChannels), amount),
            abi.encode(true)
        );
    }

    function _helperNoTokenTransferFromMock(address owner, uint256 amount) private {
        vm.mockCall(
            address(hoprToken),
            abi.encodeWithSelector(hoprToken.transferFrom.selector, owner, address(hoprChannels), amount),
            abi.encode(false)
        );
    }

    function _helperTokenTransferMock(address dest, uint256 amount) private {
        vm.mockCall(
            address(hoprToken), abi.encodeWithSelector(hoprToken.transfer.selector, dest, amount), abi.encode(true)
        );
    }

    function _helperNoTokenTransferMock(address dest, uint256 amount) private {
        vm.mockCall(
            address(hoprToken), abi.encodeWithSelector(hoprToken.transfer.selector, dest, amount), abi.encode(false)
        );
    }

    function _wrapChannel(
        uint256 balance,
        uint256 ticketIndex,
        uint256 closureTime,
        uint256 epoch,
        HoprChannelsType.ChannelStatus status
    )
        private
        pure
        returns (HoprChannelsType.Channel memory)
    {
        return HoprChannelsType.Channel(
            HoprChannelsType.Balance.wrap(uint96(balance)),
            HoprChannelsType.TicketIndex.wrap(uint48(ticketIndex)),
            HoprChannelsType.Timestamp.wrap(uint32(closureTime)),
            HoprChannelsType.ChannelEpoch.wrap(uint24(epoch)),
            status
        );
    }

    /**
     * Unwraps the channel state from a Channel struct to a compact bytes32
     *
     * @param channel The state of the channel in its struct
     * @return channelStorage The compact bytes32 format of channel State
     */
    function _unwrapChannel(HoprChannelsType.Channel memory channel) private pure returns (bytes32 channelStorage) {
        assembly {
            channelStorage := or(
                or(
                    shl(
                        96, 
                        mload(add(channel, 0x20)) // channel.ticketIndex
                    ), // left shift index by 96
                    mload(channel) // channelBalance
                ), 
                or(
                    shl(
                        144, 
                        mload(add(channel, 0x40)) // channel.closureTime
                    ), // left shift timestamp by 96 + 48
                    or(
                        shl(
                            176, 
                            mload(add(channel, 0x60)) // channel.epoch
                        ), // left shift by 96 + 48 + 32,
                        shl(
                            200,
                            mload(add(channel, 0x80)) // channel.status
                        ) // left shift by 96 + 48 + 32 + 24
                    )
                )
            )
        }
    }

    /**
     * Wraps the channel state into a Channel struct.
     *
     * @param state The compact bytes32 format of channel State
     * @return channel The state of the channel in its struct
     */
    function _expandChannel(bytes32 state) public pure returns (HoprChannelsType.Channel memory channel) {
        // Get the channel state from the storage slot
        uint256 channelBalance;
        uint256 ticketIndex;
        uint256 closureTime;
        uint256 epoch;
        uint256 status;
        assembly {
            channelBalance := and(state, 0xffffffffffffffffffffffff)
            ticketIndex := and(shr(96, state), 0xffffffffffff)
            closureTime := and(shr(144, state), 0xffffffff)
            epoch := and(shr(176, state), 0xffffff)
            status := and(shr(200, state), 0xff)
        }
        return _wrapChannel(
            channelBalance,
            ticketIndex,
            closureTime,
            epoch,
            HoprChannelsType.ChannelStatus(uint8(status))
        );
    }

    function getChannelFromTuple(address src, address dest) public view returns (HoprChannelsType.Channel memory) {
        (
            HoprChannelsType.Balance balance,
            HoprChannelsType.TicketIndex ticketIndex,
            HoprChannelsType.Timestamp closureTime,
            HoprChannelsType.ChannelEpoch epoch,
            HoprChannelsType.ChannelStatus status
        ) = hoprChannels.channels(hoprChannels._getChannelId(src, dest));
        return HoprChannelsType.Channel(balance, ticketIndex, closureTime, epoch, status);
    }
}
