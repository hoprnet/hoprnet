// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import './utils/ERC1820Registry.sol';
import './utils/Accounts.sol';
import './utils/Channels.sol';
import './utils/Tickets.sol';
import '../src/Channels.sol';
import 'forge-std/Test.sol';

contract HoprChannelsTest is
  Test,
  ERC1820RegistryFixtureTest,
  AccountsFixtureTest,
  ChannelsUtilsTest,
  TicketsUtilsTest
{
  HoprChannels public hoprChannels;
  bytes32 channelIdAB = getChannelId(accountA.accountAddr, accountB.accountAddr);
  bytes32 channelIdBA = getChannelId(accountB.accountAddr, accountA.accountAddr);

  uint256 constant ENOUGH_TIME_FOR_CLOSURE = 100;
  uint256 public MAX_USED_BALANCE;

  // uint256 public globalSnapshot;

  function setUp() public virtual override {
    super.setUp();
    // make vm.addr(1) HoprToken contract
    hoprChannels = new HoprChannels(vm.addr(1), HoprChannels.Timestamp.wrap(15));
    MAX_USED_BALANCE = uint256(HoprChannels.Balance.unwrap(hoprChannels.MAX_USED_BALANCE()));
  }

  function testFundChannelMulti(uint96 amount1, uint96 amount2) public {
    // between MIN_USED_BALANCE and MAX_USED_BALANCE
    amount1 = uint96(bound(amount1, 2, MAX_USED_BALANCE - 1));
    amount2 = uint96(bound(amount2, 2, MAX_USED_BALANCE - 1));

    // channels
    HoprChannels.Channel memory channelAB = wrapChannel(bytes32(0), amount1, 0, HoprChannels.ChannelStatus.OPEN, 1, 0);
    HoprChannels.Channel memory channelBA = wrapChannel(bytes32(0), amount2, 0, HoprChannels.ChannelStatus.OPEN, 1, 0);

    vm.prank(address(1));
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature(
        'transferFrom(address,address,uint256)',
        address(1),
        address(hoprChannels),
        amount1 + amount2
      ),
      abi.encode(true)
    );
    // fund channel for two parties triggers
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelOpened(accountA.accountAddr, accountB.accountAddr, HoprChannels.Balance.wrap(amount1));
    vm.expectEmit(true, false, false, true, address(hoprChannels));
    emit ChannelBalanceIncreased(channelIdAB, HoprChannels.Balance.wrap(amount1));
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelOpened(accountB.accountAddr, accountA.accountAddr, HoprChannels.Balance.wrap(amount2));
    vm.expectEmit(true, false, false, true, address(hoprChannels));
    emit ChannelBalanceIncreased(channelIdBA, HoprChannels.Balance.wrap(amount2));
    // fund channel A->B and B->A
    hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), accountB.accountAddr, HoprChannels.Balance.wrap(amount2));
    vm.clearMockedCalls();

    // check vallidate from channels()
    assertEqChannelsWithId(hoprChannels, channelIdAB, channelAB);
    assertEqChannelsWithId(hoprChannels, channelIdBA, channelBA);
  }

  function testFailFundChannelMulti_SameSourceAndDestination(uint96 amount1, uint96 amount2) public {
    // between MIN_USED_BALANCE and MAX_USED_BALANCE
    amount1 = uint96(bound(amount1, 2, MAX_USED_BALANCE - 1));
    amount2 = uint96(bound(amount2, 2, MAX_USED_BALANCE - 1));

    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), accountA.accountAddr, HoprChannels.Balance.wrap(amount2));
  }

  function testFailFundChannelMulti_FromSourceZero(uint96 amount1, uint96 amount2) public {
    // between MIN_USED_BALANCE and MAX_USED_BALANCE
    amount1 = uint96(bound(amount1, 2, MAX_USED_BALANCE - 1));
    amount2 = uint96(bound(amount2, 2, MAX_USED_BALANCE - 1));

    vm.prank(address(1));
    hoprChannels.fundChannelMulti(address(0), HoprChannels.Balance.wrap(amount1), accountB.accountAddr, HoprChannels.Balance.wrap(amount2));
  }

  function testFailFundChannelMulti_ToDestinationZero(uint96 amount1, uint96 amount2) public {
    // between MIN_USED_BALANCE and MAX_USED_BALANCE
    amount1 = uint96(bound(amount1, 2, MAX_USED_BALANCE - 1));
    amount2 = uint96(bound(amount2, 2, MAX_USED_BALANCE - 1));

    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), address(0), HoprChannels.Balance.wrap(amount2));
  }

  function testFailFundChannelMulti_AmountZero() public {
    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(uint96(0)), accountB.accountAddr, HoprChannels.Balance.wrap(uint96(0)));
  }

  function testFailFundChannelMulti_AmountExceedsMax(uint96 amount1, uint96 amount2) public {
    amount1 = uint96(bound(amount1, MAX_USED_BALANCE, 1e36));
    amount2 = uint96(bound(amount2, MAX_USED_BALANCE, 1e36));

    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), accountB.accountAddr, HoprChannels.Balance.wrap(amount2));
  }

  function testSetCommitment(uint96 amount1, uint96 amount2, bytes32 secret) public {
    amount1 = uint96(bound(amount1, 2, MAX_USED_BALANCE - 1));
    amount2 = uint96(bound(amount2, 2, MAX_USED_BALANCE - 1));
    vm.assume(secret != bytes32(0));

    _helperFundMultiAB(amount1, amount2);

   // channels
    HoprChannels.Channel memory channelAB = wrapChannel(bytes32(0), amount1, 0, HoprChannels.ChannelStatus.OPEN, 1, 0);
    HoprChannels.Channel memory channelBA = wrapChannel(secret, amount2, 0, HoprChannels.ChannelStatus.OPEN, 1, 0);

    vm.expectEmit(true, false, false, true, address(hoprChannels));
    emit CommitmentSet(CHANNEL_BA_ID, HoprChannels.ChannelEpoch.wrap(1));

    // accountA bump channel B->A with a non-zero secret
    vm.prank(accountA.accountAddr);
    hoprChannels.setCommitment(secret, accountB.accountAddr);

    // check vallidate from channels()
    assertEqChannelsWithId(hoprChannels, channelIdAB, channelAB);
    assertEqChannelsWithId(hoprChannels, channelIdBA, channelBA);
  }

  function testSetCommitmentWithPreCommitment(uint96 amount1, uint96 amount2, bytes32 secret) public {
    amount1 = uint96(bound(amount1, 2, MAX_USED_BALANCE - 1));
    amount2 = uint96(bound(amount2, 2, MAX_USED_BALANCE - 1));
    vm.assume(secret != bytes32(0));

    // fund channel
    _helperFundMultiAB(amount1, amount2);

    // Firstly, accountA bumps channel B->A with a non-zero secret
    vm.prank(accountA.accountAddr);
    hoprChannels.setCommitment(secret, accountB.accountAddr);

   // channels
    HoprChannels.Channel memory channelAB = wrapChannel(bytes32(0), amount1, 0, HoprChannels.ChannelStatus.OPEN, 1, 0);
    HoprChannels.Channel memory channelBA = wrapChannel(secret, amount2, 0, HoprChannels.ChannelStatus.OPEN, 1, 0);

    // check vallidate from channels()
    assertEqChannelsWithId(hoprChannels, channelIdAB, channelAB);
    assertEqChannelsWithId(hoprChannels, channelIdBA, channelBA);
  }

  // function testBumpChannelWithBothCommitments(
  //   uint96 amount1,
  //   uint96 amount2,
  //   bytes32 secret1,
  //   bytes32 secret2
  // ) public {
  //   vm.assume(amount1 > 0);
  //   vm.assume(amount2 > 0);
  //   amount1 = bound(amount1, 1, 1e36);
  //   amount2 = bound(amount2, 1, 1e36);

  //   vm.assume(secret1 != bytes32(0));
  //   vm.assume(secret2 != bytes32(0));

  //   // Both accountA and accountB bump respective channels with secrets
  //   vm.prank(accountA.accountAddr);
  //   hoprChannels.setCommitment(secret1, accountB.accountAddr);
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.setCommitment(secret2, accountA.accountAddr);

  //   // channels
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     amount1,
  //     secret2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // ticket epoch is 1
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2,
  //     secret1,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // fund channel and check emitted events
  //   _helperFundMultiAB(amount1, amount2, accountA.accountAddr, accountB.accountAddr, channelAB, channelBA);

  //   // check vallidate from channels()
  //   assertEqChannels(hoprChannels.channels(channelIdAB), channelAB);
  //   assertEqChannels(hoprChannels.channels(channelIdBA), channelBA);
  // }

  // /**
  //  *@dev it should fund A->B using send
  //  */
  // function testFundChannelABWithSend(uint256 amount) public {
  //   amount = bound(amount, 1, 1e36);
  //   // both accountA and accountB are announced
  //   _helperAnnounceAB(true, true);
  //   // expect to emit a channel
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount,
  //     bytes32(0),
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
  //     1,
  //     0
  //   );
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
  //   vm.expectEmit(true, true, true, true, address(hoprChannels));
  //   emit ChannelFunded(accountB.accountAddr, accountB.accountAddr, accountA.accountAddr, amount);

  //   // mock token contract to call `tokensReceived()` on hoprChannels contract by account B
  //   vm.prank(vm.addr(1));
  //   hoprChannels.tokensReceived(
  //     address(0),
  //     accountB.accountAddr,
  //     address(hoprChannels),
  //     amount,
  //     abi.encode(accountB.accountAddr, accountA.accountAddr, amount, HoprChannels.ChannelStatus.CLOSED),
  //     hex'00'
  //   );
  // }

  // /**
  //  * @dev With single funded HoprChannels: AB: amount1, where amount1 is above 10,
  //    it should reedem ticket for account B -> directly to wallet
  //  */
  // function testRedeemTicketWithSingleFundedHoprChannel(uint96 amount1) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   // Firstly, accountB bumps channel A->B with SECRET_2
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.setCommitment(SECRET_2, accountA.accountAddr);
  //   // then fund channel
  //   _helperFundMultiAB(amount1, 0);
  //   // mock token transfer from hoprChannels
  //   vm.prank(address(hoprChannels));
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountB.accountAddr, TICKET_AB_WIN.amount), // provide specific
  //     abi.encode(true)
  //   );

  //   // channels
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     amount1,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // ticket epoch is 1
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     0,
  //     bytes32(0),
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.CLOSED,
  //     0,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(hoprChannels.channels(channelIdAB), channelAB);
  //   assertEqChannels(hoprChannels.channels(channelIdBA), channelBA);

  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit TicketRedeemed(
  //     TICKET_AB_WIN.source,
  //     accountB.accountAddr,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // accountB redeem ticket
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.destination,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );
  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev With funded HoprChannels: AB: amount1, BA: amount 2, where both amount1 and amount2 are above 10 (amount of TICKET_AB_WIN and TICKET_BA_WIN),
  //  * it should fail to redeem ticket when ticket has been already redeemed
  //  */
  // function testFail_RedeemARedeemedTicket(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit TicketRedeemed(
  //     TICKET_AB_WIN.source,
  //     accountB.accountAddr,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // accountB redeem ticket
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.destination,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // fail to redeem the redeemed ticket
  //   vm.expectRevert(bytes('redemptions must be in order'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.destination,
  //     SECRET_0,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // fail to redeem the redeemed ticket
  //   vm.expectRevert(bytes('ticket epoch must match'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.destination,
  //     SECRET_0,
  //     TICKET_AB_WIN.ticketEpoch + 1,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to redeem ticket when signer is not the issuer
  //  */
  // function testFail_RedeemATicketFromAnotherSigner(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountB redeem ticket
  //   vm.prank(accountB.accountAddr);
  //   // fail to redeem the redeemed ticket due to wrong signature
  //   vm.expectRevert(bytes('signer must match the counterparty'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_LOSS.source,
  //     TICKET_AB_LOSS.destination,
  //     TICKET_AB_LOSS.nextCommitment,
  //     TICKET_AB_LOSS.ticketEpoch,
  //     TICKET_AB_LOSS.ticketIndex,
  //     TICKET_AB_LOSS.proofOfRelaySecret,
  //     TICKET_AB_LOSS.amount,
  //     TICKET_AB_LOSS.winProb,
  //     TICKET_AB_LOSS.signature
  //   );
  // }

  // /**
  //  * @dev With funded open channels, A can initialize channel closure
  //  */
  // function test_AInitializeChannelClosure(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // Update channel AB state
  //   HoprChannels.Channel memory channelAB = getChannelFromTuple(hoprChannels, channelIdAB);
  //   channelAB.status = HoprChannels.ChannelStatus.PENDING_TO_CLOSE;
  //   channelAB.closureTime = uint32(block.timestamp) + hoprChannels.secsClosure();

  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, channelAB);
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelClosureInitiated(accountA.accountAddr, accountB.accountAddr, uint32(block.timestamp));
  //   // account A initiate channel closure
  //   vm.prank(accountA.accountAddr);
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, accountB.accountAddr);

  //   // channels after ticket redemption
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(hoprChannels.channels(channelIdAB), channelAB);
  //   assertEqChannels(hoprChannels.channels(channelIdBA), channelBA);
  // }

  // /**
  //  * @dev With funded open channels, B can initialize channel closure
  //  */
  // function test_BInitializeChannelClosure(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // Update channel BA state
  //   HoprChannels.Channel memory channelBA = getChannelFromTuple(hoprChannels, channelIdBA);
  //   channelBA.status = HoprChannels.ChannelStatus.PENDING_TO_CLOSE;
  //   channelBA.closureTime = uint32(block.timestamp) + hoprChannels.secsClosure();

  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
  //   // account B initiate channel closure
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.initiateChannelClosure(accountB.accountAddr, accountA.accountAddr);

  //   // channels after ticket redemption
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     amount1,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(hoprChannels.channels(channelIdAB), channelAB);
  //   assertEqChannels(hoprChannels.channels(channelIdBA), channelBA);
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to redeem ticket if it's a loss
  //  */
  // function testFail_RedeemLossTicket(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountB redeem ticket
  //   vm.prank(accountB.accountAddr);
  //   // fail to redeem the redeemed ticket due to wrong signature
  //   vm.expectRevert(bytes('ticket must be a win'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.destination,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     hex'2bea87a4a771731c3dcf8c17443765344a0bfee9354f87636d068055cde15f6e2ddb4ce35e11b57dcc44f28206af89894feea677834c6e5be17d180cbeb514ba' // use AccountB to sign `TICKET_AB_WIN`
  //   );
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to initialize channel closure A->A
  //  */
  // function testRevert_InitiateChannelClosureCircularChannel(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountA redeem ticket
  //   vm.prank(accountA.accountAddr);
  //   // fail to initiate channel closure for channel pointing to the source
  //   vm.expectRevert(bytes('source and destination must not be the same'));
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, accountA.accountAddr);
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to initialize channel closure A->0
  //  */
  // function testRevert_InitiateChannelClosureFromAddressZero(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountA redeem ticket
  //   vm.prank(accountA.accountAddr);
  //   // fail to initiate channel closure pointing to address zero
  //   vm.expectRevert(bytes('destination must not be empty'));
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, address(0));
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to finalize channel closure when is not pending
  //  */
  // function testRevert_FinalizedNotInitiatedChannelClosure(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountA redeem ticket
  //   vm.prank(accountA.accountAddr);
  //   // fail to force finalize channel closure
  //   vm.expectRevert(bytes('channel must be pending to close'));
  //   hoprChannels.finalizeChannelClosure(accountA.accountAddr, accountB.accountAddr);
  // }

  // /**
  //  * @dev With funded non-open channels:
  //  * it should fail to initialize channel closure when channel is not open
  //  */
  // function testRevert_InitializedClosureForNonOpenChannel(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // fund channels
  //   _helperFundMultiAB(amount1, amount2);
  //   // accountA redeem ticket
  //   vm.prank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, accountB.accountAddr);

  //   // fail to force initiate again channel closure
  //   vm.expectRevert(bytes('channel must be open or waiting for commitment'));
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, accountB.accountAddr);
  // }

  // /**
  //  * @dev With funded non-open channels:
  //  * it should finalize channel closure
  //  */
  // function test_FinalizeInitializedClosure(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // fund channels
  //   _helperFundMultiAB(amount1, amount2);
  //   // accountA redeem ticket
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, accountB.accountAddr);

  //   // increase enough time for channel closure;
  //   vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);

  //   // Update channel AB state
  //   HoprChannels.Channel memory channelAB = getChannelFromTuple(hoprChannels, channelIdAB);
  //   // succeed in finalizing channel closure
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelClosureFinalized(accountA.accountAddr, accountB.accountAddr, channelAB.closureTime, amount1);
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(
  //     accountA.accountAddr,
  //     accountB.accountAddr,
  //     HoprChannels.Channel(0, bytes32(0), 0, 0, HoprChannels.ChannelStatus.CLOSED, 1, 0)
  //   );
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, amount1), // provide specific
  //     abi.encode(true)
  //   );
  //   hoprChannels.finalizeChannelClosure(accountA.accountAddr, accountB.accountAddr);
  //   vm.stopPrank();
  // }

  // /**
  //  * @dev With funded non-open channels:
  //  * it should fail to initialize channel closure, in various situtations
  //  */
  // function testRevert_FinalizeChannelClosure(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // fund channels
  //   _helperFundMultiAB(amount1, amount2);
  //   // accountA redeem ticket
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, accountB.accountAddr);

  //   // fail when source and destination are the same
  //   vm.expectRevert(bytes('source and destination must not be the same'));
  //   hoprChannels.finalizeChannelClosure(accountA.accountAddr, accountA.accountAddr);

  //   // fail when the destination is empty
  //   vm.expectRevert(bytes('destination must not be empty'));
  //   hoprChannels.finalizeChannelClosure(accountA.accountAddr, address(0));

  //   // fail when finallization hasn't reached yet
  //   vm.expectRevert(bytes('closureTime must be before now'));
  //   hoprChannels.finalizeChannelClosure(accountA.accountAddr, accountB.accountAddr);
  //   vm.stopPrank();
  // }

  // /**
  //  * @dev With a closed channel:
  //  * it should fail to redeem ticket when channel in closed
  //  */
  // function testRevert_WhenRedeemTicketsInAClosedChannel(uint96 amount1, uint96 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // with a closed channel A->B
  //   _helperWithAClosedChannel(amount1, amount2);

  //   vm.prank(accountB.accountAddr);
  //   // fail when source and destination are the same
  //   vm.expectRevert(bytes('spending channel must be open or pending to close'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.destination,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );
  // }

  // /**
  //  * @dev With a closed channel:
  //  * it should allow a fund to reopen channel
  //  */
  // function test_AllowFundToReopenAClosedChannel(
  //   uint96 amount1,
  //   uint96 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than 0
  //   reopenAmount1 = bound(reopenAmount1, 1, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);

  //   // with a closed channel A->B
  //   _helperWithAClosedChannel(amount1, amount2);

  //   // fund channel again
  //   vm.prank(address(1));
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       reopenAmount1 + reopenAmount2
  //     ),
  //     abi.encode(true)
  //   );
  //   // succeed in reopening a channel
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, reopenAmount1, reopenAmount2);

  //   // succeed in reopening a channel
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     reopenAmount1,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     2,
  //     0
  //   );

  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2 + reopenAmount2,
  //     hex'00', // never bumped
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(hoprChannels.channels(channelIdAB), channelAB);
  //   assertEqChannels(hoprChannels.channels(channelIdBA), channelBA);

  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev With a reopened channel:
  //  * it should pass the sanity check
  //  */
  // function test_SanityCheck(uint96 amount1, uint96 amount2, uint256 reopenAmount1, uint256 reopenAmount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than 0
  //   reopenAmount1 = bound(reopenAmount1, 1, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);
  //   _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

  //   // succeed in reopening a channel
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     reopenAmount1,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     2,
  //     0
  //   );

  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2 + reopenAmount2,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(hoprChannels.channels(channelIdAB), channelAB);
  //   assertEqChannels(hoprChannels.channels(channelIdBA), channelBA);
  // }

  // /**
  //  * @dev With a reopened channel:
  //  * it should fail to redeem ticket when channel in in different channelEpoch
  //  */
  // function testRevert_WhenDifferentChannelEpochFailToRedeemTicket(
  //   uint96 amount1,
  //   uint96 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than 0
  //   reopenAmount1 = bound(reopenAmount1, 1, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);
  //   _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

  //   vm.prank(accountB.accountAddr);
  //   // fail when source and destination are the same
  //   vm.expectRevert(bytes('signer must match the counterparty'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.destination,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );
  // }

  // /**
  //  * @dev With a reopened channel:
  //  * it should should reedem ticket for account B
  //  */
  // function test_RedeemTicketForAccountB(
  //   uint96 amount1,
  //   uint96 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than TICKET_AB_WIN_RECYCLED.amount and 0 respectively
  //   reopenAmount1 = bound(reopenAmount1, TICKET_AB_WIN_RECYCLED.amount, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);
  //   _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

  //   vm.startPrank(accountB.accountAddr);
  //   // allow token transfer
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', TICKET_AB_WIN_RECYCLED.destination, TICKET_AB_WIN_RECYCLED.amount), // provide specific
  //     abi.encode(true)
  //   );
  //   // fail when source and destination are the same
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN_RECYCLED.source,
  //     TICKET_AB_WIN_RECYCLED.destination,
  //     TICKET_AB_WIN_RECYCLED.nextCommitment,
  //     TICKET_AB_WIN_RECYCLED.ticketEpoch,
  //     TICKET_AB_WIN_RECYCLED.ticketIndex,
  //     TICKET_AB_WIN_RECYCLED.proofOfRelaySecret,
  //     TICKET_AB_WIN_RECYCLED.amount,
  //     TICKET_AB_WIN_RECYCLED.winProb,
  //     TICKET_AB_WIN_RECYCLED.signature
  //   );

  //   // succeed in redeeming the ticket
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     reopenAmount1 - TICKET_AB_WIN_RECYCLED.amount,
  //     SECRET_1,
  //     0,
  //     1,
  //     HoprChannels.ChannelStatus.OPEN,
  //     2,
  //     0
  //   );

  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2 + reopenAmount2 + TICKET_AB_WIN_RECYCLED.amount,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(hoprChannels.channels(channelIdAB), channelAB);
  //   // assertEqChannels(hoprChannels.channels(channelIdBA), channelBA);
  //   vm.stopPrank();
  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev With a reopened channel:
  //  * it should allow closure
  //  */
  // function test_AllowClosingReopenedChannel(
  //   uint96 amount1,
  //   uint96 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than TICKET_AB_WIN_RECYCLED.amount and 0 respectively
  //   reopenAmount1 = bound(reopenAmount1, TICKET_AB_WIN_RECYCLED.amount, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);
  //   _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

  //   // accountA initiate channel closure
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, accountB.accountAddr);
  //   // increase enough time for channel closure;
  //   vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);
  //   // finalize channel closure
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, reopenAmount1), // provide specific
  //     abi.encode(true)
  //   );
  //   hoprChannels.finalizeChannelClosure(accountA.accountAddr, accountB.accountAddr);
  //   vm.stopPrank();

  //   // succeed in redeeming the ticket
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     0,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.CLOSED,
  //     2,
  //     0
  //   );

  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2 + reopenAmount2,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(hoprChannels.channels(channelIdAB), channelAB);
  //   assertEqChannels(hoprChannels.channels(channelIdBA), channelBA);
  // }

  /**
   *@dev Helper function to fund channel A->B and B->A with `fundChannelMulti`
   */
  function _helperFundMultiAB(uint96 amount1, uint96 amount2) internal {
    vm.prank(address(1));
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature(
        'transferFrom(address,address,uint256)',
        address(1),
        address(hoprChannels),
        amount1 + amount2
      ),
      abi.encode(true)
    );
    // fund channel A->B and B->A
    hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), accountB.accountAddr, HoprChannels.Balance.wrap(amount2));
    vm.clearMockedCalls();
  }

  // /**
  //  * @dev Helper function to fund channel A->B (amount1) and B->A (amount2) to OPEN,
  //  * where both amount1 and amount2 are above the amount of possible winning ticket
  //  * (i.e. TICKET_AB_WIN and TICKET_BA_WIN)
  //  */
  // function _helperOpenBidirectionalChannels(uint96 amount1, uint96 amount2) internal {
  //   // accountB bumps channel A->B with SECRET_2
  //   // accountA bumps channel B->A with SECRET_2
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.setCommitment(SECRET_2, accountA.accountAddr);
  //   vm.prank(accountA.accountAddr);
  //   hoprChannels.setCommitment(SECRET_2, accountB.accountAddr);
  //   // then fund channel
  //   _helperFundMultiAB(amount1, amount2);
  // }

  // /**
  //  * @dev With a closed channel:
  //  */
  // function _helperWithAClosedChannel(uint96 amount1, uint96 amount2) public {
  //   // make channel A->B open
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.setCommitment(SECRET_2, accountA.accountAddr);
  //   // fund channels
  //   _helperFundMultiAB(amount1, amount2);

  //   // accountA initiate channel closure
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, accountB.accountAddr);
  //   // increase enough time for channel closure;
  //   vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);
  //   // finalize channel closure
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, amount1), // provide specific
  //     abi.encode(true)
  //   );
  //   hoprChannels.finalizeChannelClosure(accountA.accountAddr, accountB.accountAddr);
  //   vm.stopPrank();
  // }

  // /**
  //  * @dev With a reopened channel:
  //  */
  // function _helperWithAReopenedChannel(
  //   uint96 amount1,
  //   uint96 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // make channel A->B and B=>A open
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountA initiate channel closure
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr, accountB.accountAddr);
  //   // increase enough time for channel closure;
  //   vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);
  //   // finalize channel closure
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, amount1), // provide specific
  //     abi.encode(true)
  //   );
  //   hoprChannels.finalizeChannelClosure(accountA.accountAddr, accountB.accountAddr);
  //   vm.stopPrank();

  //   // fund channel again
  //   vm.prank(address(1));
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       reopenAmount1 + reopenAmount2
  //     ),
  //     abi.encode(true)
  //   );
  //   // succeed in reopening a channel
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, reopenAmount1, reopenAmount2);
  //   vm.clearMockedCalls();
  // }
}
