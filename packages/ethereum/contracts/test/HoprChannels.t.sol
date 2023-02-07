// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import './utils/ERC1820Registry.sol';
import './utils/Accounts.sol';
import './utils/Channels.sol';
import './utils/Tickets.sol';
import '../src/HoprChannels.sol';
import './mocks/ChannelsMock.sol';
import 'forge-std/Test.sol';

contract HoprChannelsTest is
  Test,
  ERC1820RegistryFixtureTest,
  AccountsFixtureTest,
  ChannelsUtilsTest,
  TicketsUtilsTest
{
  HoprChannels public hoprChannels;
  ChannelsMockTest public channelsMockTest;
  bytes32 channelIdAB = getChannelId(accountA.accountAddr, accountB.accountAddr);
  bytes32 channelIdBA = getChannelId(accountB.accountAddr, accountA.accountAddr);

  uint256 constant ENOUGH_TIME_FOR_CLOSURE = 100;

  // uint256 public globalSnapshot;

  function setUp() public virtual override {
    super.setUp();
    // make vm.addr(1) HoprToken contract
    hoprChannels = new HoprChannels(vm.addr(1), 15);
    channelsMockTest = new ChannelsMockTest(vm.addr(1), 15);
  }

  function testAnnounceAddressFromPublicKey() public {
    bytes memory multiAddress = hex'1234';
    vm.prank(accountA.accountAddr);
    vm.expectEmit(true, false, false, true, address(hoprChannels));
    emit Announcement(accountA.accountAddr, accountA.publicKey, multiAddress);
    hoprChannels.announce(accountA.publicKey, multiAddress);
  }

  function testRevert_AnnouceWrongPublicKey() public {
    bytes memory multiAddress = hex'1234';
    vm.prank(accountB.accountAddr);
    vm.expectRevert("publicKey's address does not match senders");
    hoprChannels.announce(accountA.publicKey, multiAddress);
  }

  function testRevert_AnnouceRandomPublicKey(bytes calldata randomPublicKey) public {
    vm.assume(keccak256(randomPublicKey) != keccak256(accountB.publicKey));
    bytes memory multiAddress = hex'1234';
    vm.prank(accountB.accountAddr);
    vm.expectRevert("publicKey's address does not match senders");
    hoprChannels.announce(randomPublicKey, multiAddress);
  }

  // // it should fail to fund without accountA announcement
  function testRevert_FundChannelMultiWithoutAccountAAnnoucement(uint256 amount1) public {
    amount1 = bound(amount1, 1, 1e36);
    // accountA is not annouced and only accountB is announced
    _helperAnnounceAB(false, true);

    vm.prank(address(1));
    vm.expectRevert('source has not announced');
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, 0);
  }

  // it should fail to fund without accountB announcement
  function testRevert_FundChannelMultiWithoutAccountBAnnoucement(uint256 amount1) public {
    amount1 = bound(amount1, 1, 1e36);
    // accountB is not annouced and only accountA is announced
    _helperAnnounceAB(true, false);

    vm.prank(address(1));
    vm.expectRevert('destination has not announced');
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, 0);
  }

  function testFundChannelMulti(uint256 amount1, uint256 amount2) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );

    // fund channel for two parties triggers
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, channelAB);
    vm.expectEmit(true, true, true, true, address(hoprChannels));
    emit ChannelFunded(address(1), accountA.accountAddr, accountB.accountAddr, amount1);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
    vm.expectEmit(true, true, true, true, address(hoprChannels));
    emit ChannelFunded(address(1), accountB.accountAddr, accountA.accountAddr, amount2);

    // announc account A and B. Fund multi for A->B and B->A
    _helperFundMultiAB(amount1, amount2);
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  function testFailFundChannelMulti_SameSourceAndDestination(uint256 amount1, uint256 amount2) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    // accountA is announced
    _helperAnnounceAB(true, false);
    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountA.accountAddr, amount1, amount2);
  }

  function testFailFundChannelMulti_FromSourceZero(uint256 amount1, uint256 amount2) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    // accountA is announced
    _helperAnnounceAB(true, false);
    vm.prank(address(1));
    hoprChannels.fundChannelMulti(address(0), accountA.accountAddr, amount1, amount2);
  }

  function testFailFundChannelMulti_ToDestinationZero(uint256 amount1, uint256 amount2) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    // accountA is announced
    _helperAnnounceAB(true, false);
    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, address(0), amount1, amount2);
  }

  function testFailFundChannelMulti_AmountZero() public {
    // both accountA and accountB are announced
    _helperAnnounceAB(true, true);
    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, address(0), 0, 0);
  }

  function testBumpChannel(
    uint256 amount1,
    uint256 amount2,
    bytes32 secret
  ) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    vm.assume(secret != bytes32(0));

    _helperFundMultiAB(amount1, amount2);

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      secret,
      1,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelBumped(accountB.accountAddr, accountA.accountAddr, secret, 1, amount2);

    // accountA bump channel B->A with a non-zero secret
    vm.prank(accountA.accountAddr);
    hoprChannels.bumpChannel(accountB.accountAddr, secret);

    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  function testBumpChannelWithPreCommitment(
    uint256 amount1,
    uint256 amount2,
    bytes32 secret
  ) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    vm.assume(secret != bytes32(0));
    // Firstly, accountA bumps channel B->A with a non-zero secret
    vm.prank(accountA.accountAddr);
    hoprChannels.bumpChannel(accountB.accountAddr, secret);

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      secret,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // order of logs matters
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, channelAB);
    vm.expectEmit(true, false, false, false, address(hoprChannels));
    emit ChannelOpened(accountB.accountAddr, accountA.accountAddr);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);

    // then fund channel
    _helperFundMultiAB(amount1, amount2);

    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  function testBumpChannelWithBothCommitments(
    uint256 amount1,
    uint256 amount2,
    bytes32 secret1,
    bytes32 secret2
  ) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    vm.assume(secret1 != bytes32(0));
    vm.assume(secret2 != bytes32(0));

    // Both accountA and accountB bump respective channels with secrets
    vm.prank(accountA.accountAddr);
    hoprChannels.bumpChannel(accountB.accountAddr, secret1);
    vm.prank(accountB.accountAddr);
    hoprChannels.bumpChannel(accountA.accountAddr, secret2);

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      secret2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      secret1,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // order of logs matters
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, channelAB);
    vm.expectEmit(true, false, false, false, address(hoprChannels));
    emit ChannelOpened(accountB.accountAddr, accountA.accountAddr);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);

    // then fund channel
    _helperFundMultiAB(amount1, amount2);

    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  /**
   *@dev it should fund A->B using send
   */
  function testFundChannelABWithSend(uint256 amount) public {
    amount = bound(amount, 1, 1e36);
    // both accountA and accountB are announced
    _helperAnnounceAB(true, true);
    // expect to emit a channel
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
    vm.expectEmit(true, true, true, true, address(hoprChannels));
    emit ChannelFunded(accountB.accountAddr, accountB.accountAddr, accountA.accountAddr, amount);

    // mock token contract to call `tokensReceived()` on hoprChannels contract by account B
    vm.prank(vm.addr(1));
    hoprChannels.tokensReceived(
      address(0),
      accountB.accountAddr,
      address(hoprChannels),
      amount,
      abi.encode(accountB.accountAddr, accountA.accountAddr, amount, HoprChannels.ChannelStatus.CLOSED),
      hex'00'
    );
  }

  /**
   * @dev With single funded HoprChannels: AB: amount1, where amount1 is above 10,
     it should reedem ticket for account B -> directly to wallet
   */
  function testRedeemTicketWithSingleFundedHoprChannel(uint256 amount1) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    // Firstly, accountB bumps channel A->B with SECRET_2
    vm.prank(accountB.accountAddr);
    hoprChannels.bumpChannel(accountA.accountAddr, SECRET_2);
    // then fund channel
    _helperFundMultiAB(amount1, 0);
    // mock token transfer from hoprChannels
    vm.prank(address(hoprChannels));
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature('transfer(address,uint256)', accountB.accountAddr, TICKET_AB_WIN.amount), // provide specific
      abi.encode(true)
    );

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      0,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.CLOSED,
      0,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);

    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit TicketRedeemed(
      TICKET_AB_WIN.source,
      accountB.accountAddr,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // accountB redeem ticket
    vm.prank(accountB.accountAddr);
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );
    vm.clearMockedCalls();
  }

  /**
   * @dev With funded HoprChannels: AB: amount1, BA: amount 2,
   * where both amount1 and amount2 are above 10,
   * it should redeem ticket for account A
   * it should redeem ticket for account B
   */
  function testRedeemTicketWithDoubleFundedHoprChannel(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    // channels before ticket redemption
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);

    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit TicketRedeemed(
      TICKET_AB_WIN.source,
      accountB.accountAddr,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // create a snapshot
    uint256 snapshotBeforeAccountBRedeemTicket = vm.snapshot();

    // accountB redeem ticket
    vm.prank(accountB.accountAddr);
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // channels after ticket redemption
    HoprChannels.Channel memory channelABAfterBRedeems = HoprChannels.Channel(
      amount1 - TICKET_AB_WIN.amount,
      SECRET_1,
      0,
      1,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBAAfterRedeems = HoprChannels.Channel(
      amount2 + TICKET_AB_WIN.amount,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelABAfterBRedeems);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBAAfterRedeems);

    // reset to snapshot
    vm.revertTo(snapshotBeforeAccountBRedeemTicket);
    // accountA redeem ticket
    vm.prank(accountA.accountAddr);
    hoprChannels.redeemTicket(
      TICKET_BA_WIN.source,
      TICKET_BA_WIN.nextCommitment,
      TICKET_BA_WIN.ticketEpoch,
      TICKET_BA_WIN.ticketIndex,
      TICKET_BA_WIN.proofOfRelaySecret,
      TICKET_BA_WIN.amount,
      TICKET_BA_WIN.winProb,
      TICKET_BA_WIN.signature
    );

    // channels after ticket redemption
    HoprChannels.Channel memory channelBAAfterARedeems = HoprChannels.Channel(
      amount2 - TICKET_BA_WIN.amount,
      SECRET_1,
      0,
      1,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelABAfterARedeems = HoprChannels.Channel(
      amount1 + TICKET_BA_WIN.amount,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelABAfterARedeems);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBAAfterARedeems);
  }

  /**
   * @dev With funded HoprChannels: AB: amount1, BA: amount 2, where both amount1 and amount2 are above 10 (amount of TICKET_AB_WIN and TICKET_BA_WIN),
   * it should fail to redeem ticket when ticket has been already redeemed
   */
  function testFail_RedeemARedeemedTicket(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit TicketRedeemed(
      TICKET_AB_WIN.source,
      accountB.accountAddr,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // accountB redeem ticket
    vm.prank(accountB.accountAddr);
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // fail to redeem the redeemed ticket
    vm.expectRevert(bytes('redemptions must be in order'));
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      SECRET_0,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // fail to redeem the redeemed ticket
    vm.expectRevert(bytes('ticket epoch must match'));
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      SECRET_0,
      TICKET_AB_WIN.ticketEpoch + 1,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );
  }

  /**
   * @dev With funded open channels:
   * it should fail to redeem ticket when signer is not the issuer
   */
  function testFail_RedeemATicketFromAnotherSigner(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    // accountB redeem ticket
    vm.prank(accountB.accountAddr);
    // fail to redeem the redeemed ticket due to wrong signature
    vm.expectRevert(bytes('signer must match the counterparty'));
    hoprChannels.redeemTicket(
      TICKET_AB_LOSS.source,
      TICKET_AB_LOSS.nextCommitment,
      TICKET_AB_LOSS.ticketEpoch,
      TICKET_AB_LOSS.ticketIndex,
      TICKET_AB_LOSS.proofOfRelaySecret,
      TICKET_AB_LOSS.amount,
      TICKET_AB_LOSS.winProb,
      TICKET_AB_LOSS.signature
    );
  }

  /**
   * @dev With funded open channels, A can initialize channel closure
   */
  function test_AInitializeChannelClosure(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    // Update channel AB state
    HoprChannels.Channel memory channelAB = getChannelFromTuple(hoprChannels, channelIdAB);
    channelAB.status = HoprChannels.ChannelStatus.PENDING_TO_CLOSE;
    channelAB.closureTime = uint32(block.timestamp) + hoprChannels.secsClosure();

    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, channelAB);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelClosureInitiated(accountA.accountAddr, accountB.accountAddr, uint32(block.timestamp));
    // account A initiate channel closure
    vm.prank(accountA.accountAddr);
    hoprChannels.initiateChannelClosure(accountB.accountAddr);

    // channels after ticket redemption
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  /**
   * @dev With funded open channels, B can initialize channel closure
   */
  function test_BInitializeChannelClosure(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    // Update channel BA state
    HoprChannels.Channel memory channelBA = getChannelFromTuple(hoprChannels, channelIdBA);
    channelBA.status = HoprChannels.ChannelStatus.PENDING_TO_CLOSE;
    channelBA.closureTime = uint32(block.timestamp) + hoprChannels.secsClosure();

    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
    // account B initiate channel closure
    vm.prank(accountB.accountAddr);
    hoprChannels.initiateChannelClosure(accountA.accountAddr);

    // channels after ticket redemption
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  /**
   * @dev With funded open channels:
   * it should fail to redeem ticket if it's a loss
   */
  function testFail_RedeemLossTicket(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    // accountB redeem ticket
    vm.prank(accountB.accountAddr);
    // fail to redeem the redeemed ticket due to wrong signature
    vm.expectRevert(bytes('ticket must be a win'));
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      hex'2bea87a4a771731c3dcf8c17443765344a0bfee9354f87636d068055cde15f6e2ddb4ce35e11b57dcc44f28206af89894feea677834c6e5be17d180cbeb514ba' // use AccountB to sign `TICKET_AB_WIN`
    );
  }

  /**
   * @dev With funded open channels:
   * it should fail to initialize channel closure A->A
   */
  function testRevert_InitiateChannelClosureCircularChannel(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    // accountA redeem ticket
    vm.prank(accountA.accountAddr);
    // fail to initiate channel closure for channel pointing to the source
    vm.expectRevert(bytes('source and destination must not be the same'));
    hoprChannels.initiateChannelClosure(accountA.accountAddr);
  }

  /**
   * @dev With funded open channels:
   * it should fail to initialize channel closure A->0
   */
  function testRevert_InitiateChannelClosureFromAddressZero(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    // accountA redeem ticket
    vm.prank(accountA.accountAddr);
    // fail to initiate channel closure pointing to address zero
    vm.expectRevert(bytes('destination must not be empty'));
    hoprChannels.initiateChannelClosure(address(0));
  }

  /**
   * @dev With funded open channels:
   * it should fail to finalize channel closure when is not pending
   */
  function testRevert_FinalizedNotInitiatedChannelClosure(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    // accountA redeem ticket
    vm.prank(accountA.accountAddr);
    // fail to force finalize channel closure
    vm.expectRevert(bytes('channel must be pending to close'));
    hoprChannels.finalizeChannelClosure(accountB.accountAddr);
  }

  /**
   * @dev With funded non-open channels:
   * it should fail to initialize channel closure when channel is not open
   */
  function testRevert_InitializedClosureForNonOpenChannel(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // fund channels
    _helperFundMultiAB(amount1, amount2);
    // accountA redeem ticket
    vm.prank(accountA.accountAddr);
    // initiate channel closure first
    hoprChannels.initiateChannelClosure(accountB.accountAddr);

    // fail to force initiate again channel closure
    vm.expectRevert(bytes('channel must be open or waiting for commitment'));
    hoprChannels.initiateChannelClosure(accountB.accountAddr);
  }

  /**
   * @dev With funded non-open channels:
   * it should finalize channel closure
   */
  function test_FinalizeInitializedClosure(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // fund channels
    _helperFundMultiAB(amount1, amount2);
    // accountA redeem ticket
    vm.startPrank(accountA.accountAddr);
    // initiate channel closure first
    hoprChannels.initiateChannelClosure(accountB.accountAddr);

    // increase enough time for channel closure;
    vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);

    // Update channel AB state
    HoprChannels.Channel memory channelAB = getChannelFromTuple(hoprChannels, channelIdAB);
    // succeed in finalizing channel closure
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelClosureFinalized(accountA.accountAddr, accountB.accountAddr, channelAB.closureTime, amount1);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(
      accountA.accountAddr,
      accountB.accountAddr,
      HoprChannels.Channel(0, bytes32(0), 0, 0, HoprChannels.ChannelStatus.CLOSED, 1, 0)
    );
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, amount1), // provide specific
      abi.encode(true)
    );
    hoprChannels.finalizeChannelClosure(accountB.accountAddr);
    vm.stopPrank();
  }

  /**
   * @dev With funded non-open channels:
   * it should fail to initialize channel closure, in various situtations
   */
  function testRevert_FinalizeChannelClosure(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // fund channels
    _helperFundMultiAB(amount1, amount2);
    // accountA redeem ticket
    vm.startPrank(accountA.accountAddr);
    // initiate channel closure first
    hoprChannels.initiateChannelClosure(accountB.accountAddr);

    // fail when source and destination are the same
    vm.expectRevert(bytes('source and destination must not be the same'));
    hoprChannels.finalizeChannelClosure(accountA.accountAddr);

    // fail when the destination is empty
    vm.expectRevert(bytes('destination must not be empty'));
    hoprChannels.finalizeChannelClosure(address(0));

    // fail when finallization hasn't reached yet
    vm.expectRevert(bytes('closureTime must be before now'));
    hoprChannels.finalizeChannelClosure(accountB.accountAddr);
    vm.stopPrank();
  }

  /**
   * @dev With a closed channel:
   * it should fail to redeem ticket when channel in closed
   */
  function testRevert_WhenRedeemTicketsInAClosedChannel(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // with a closed channel A->B
    _helperWithAClosedChannel(amount1, amount2);

    vm.prank(accountB.accountAddr);
    // fail when source and destination are the same
    vm.expectRevert(bytes('spending channel must be open or pending to close'));
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );
  }

  /**
   * @dev With a closed channel:
   * it should allow a fund to reopen channel
   */
  function test_AllowFundToReopenAClosedChannel(
    uint256 amount1,
    uint256 amount2,
    uint256 reopenAmount1,
    uint256 reopenAmount2
  ) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // reopen amount should be larger than 0
    reopenAmount1 = bound(reopenAmount1, 1, 1e36);
    reopenAmount2 = bound(reopenAmount2, 1, 1e36);

    // with a closed channel A->B
    _helperWithAClosedChannel(amount1, amount2);

    // fund channel again
    vm.prank(address(1));
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature(
        'transferFrom(address,address,uint256)',
        address(1),
        address(hoprChannels),
        reopenAmount1 + reopenAmount2
      ),
      abi.encode(true)
    );
    // succeed in reopening a channel
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, reopenAmount1, reopenAmount2);

    // succeed in reopening a channel
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      reopenAmount1,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      2,
      0
    );

    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2 + reopenAmount2,
      hex'00', // never bumped
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);

    vm.clearMockedCalls();
  }

  /**
   * @dev With a reopened channel:
   * it should pass the sanity check
   */
  function test_SanityCheck(
    uint256 amount1,
    uint256 amount2,
    uint256 reopenAmount1,
    uint256 reopenAmount2
  ) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // reopen amount should be larger than 0
    reopenAmount1 = bound(reopenAmount1, 1, 1e36);
    reopenAmount2 = bound(reopenAmount2, 1, 1e36);
    _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

    // succeed in reopening a channel
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      reopenAmount1,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      2,
      0
    );

    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2 + reopenAmount2,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  /**
   * @dev With a reopened channel:
   * it should fail to redeem ticket when channel in in different channelEpoch
   */
  function testRevert_WhenDifferentChannelEpochFailToRedeemTicket(
    uint256 amount1,
    uint256 amount2,
    uint256 reopenAmount1,
    uint256 reopenAmount2
  ) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // reopen amount should be larger than 0
    reopenAmount1 = bound(reopenAmount1, 1, 1e36);
    reopenAmount2 = bound(reopenAmount2, 1, 1e36);
    _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

    vm.prank(accountB.accountAddr);
    // fail when source and destination are the same
    vm.expectRevert(bytes('signer must match the counterparty'));
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );
  }

  /**
   * @dev With a reopened channel:
   * it should should reedem ticket for account B
   */
  function test_RedeemTicketForAccountB(
    uint256 amount1,
    uint256 amount2,
    uint256 reopenAmount1,
    uint256 reopenAmount2
  ) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // reopen amount should be larger than TICKET_AB_WIN_RECYCLED.amount and 0 respectively
    reopenAmount1 = bound(reopenAmount1, TICKET_AB_WIN_RECYCLED.amount, 1e36);
    reopenAmount2 = bound(reopenAmount2, 1, 1e36);
    _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

    vm.prank(accountB.accountAddr);
    // fail when source and destination are the same
    hoprChannels.redeemTicket(
      TICKET_AB_WIN_RECYCLED.source,
      TICKET_AB_WIN_RECYCLED.nextCommitment,
      TICKET_AB_WIN_RECYCLED.ticketEpoch,
      TICKET_AB_WIN_RECYCLED.ticketIndex,
      TICKET_AB_WIN_RECYCLED.proofOfRelaySecret,
      TICKET_AB_WIN_RECYCLED.amount,
      TICKET_AB_WIN_RECYCLED.winProb,
      TICKET_AB_WIN_RECYCLED.signature
    );

    // succeed in redeeming the ticket
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      reopenAmount1 - TICKET_AB_WIN_RECYCLED.amount,
      SECRET_1,
      0,
      1,
      HoprChannels.ChannelStatus.OPEN,
      2,
      0
    );

    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2 + reopenAmount2 + TICKET_AB_WIN_RECYCLED.amount,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  /**
   * @dev With a reopened channel:
   * it should allow closure
   */
  function test_AllowClosingReopenedChannel(
    uint256 amount1,
    uint256 amount2,
    uint256 reopenAmount1,
    uint256 reopenAmount2
  ) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // reopen amount should be larger than TICKET_AB_WIN_RECYCLED.amount and 0 respectively
    reopenAmount1 = bound(reopenAmount1, TICKET_AB_WIN_RECYCLED.amount, 1e36);
    reopenAmount2 = bound(reopenAmount2, 1, 1e36);
    _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

    // accountA initiate channel closure
    vm.startPrank(accountA.accountAddr);
    // initiate channel closure first
    hoprChannels.initiateChannelClosure(accountB.accountAddr);
    // increase enough time for channel closure;
    vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);
    // finalize channel closure
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, reopenAmount1), // provide specific
      abi.encode(true)
    );
    hoprChannels.finalizeChannelClosure(accountB.accountAddr);
    vm.stopPrank();

    // succeed in redeeming the ticket
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      0,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.CLOSED,
      2,
      0
    );

    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2 + reopenAmount2,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  /**
   * @dev test internals with mock:
   * it should get channel id, returning the same channel ID
   */
  function test_SameChannelId() public {
    assertEq(channelIdAB, channelsMockTest.getChannelIdInternal(accountA.accountAddr, accountB.accountAddr));
  }

  /**
   *@dev Helper function to announce account A and B
   */
  function _helperAnnounceAB(bool annouceA, bool announceB) internal {
    // both accountA and accountB are announced
    bytes memory multiAddress = hex'1234';
    if (annouceA) {
      vm.prank(accountA.accountAddr);
      hoprChannels.announce(accountA.publicKey, multiAddress);
    }
    if (announceB) {
      vm.prank(accountB.accountAddr);
      hoprChannels.announce(accountB.publicKey, multiAddress);
    }
  }

  /**
   *@dev Helper function to fund channel A->B and B->A with `fundChannelMulti`
   */
  function _helperFundMultiAB(uint256 amount1, uint256 amount2) internal {
    // both accountA and accountB are announced
    _helperAnnounceAB(true, true);
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
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, amount2);
    vm.clearMockedCalls();
  }

  /**
   * @dev Helper function to fund channel A->B (amount1) and B->A (amount2) to OPEN,
   * where both amount1 and amount2 are above the amount of possible winning ticket
   * (i.e. TICKET_AB_WIN and TICKET_BA_WIN)
   */
  function _helperOpenBidirectionalChannels(uint256 amount1, uint256 amount2) internal {
    // accountB bumps channel A->B with SECRET_2
    // accountA bumps channel B->A with SECRET_2
    vm.prank(accountB.accountAddr);
    hoprChannels.bumpChannel(accountA.accountAddr, SECRET_2);
    vm.prank(accountA.accountAddr);
    hoprChannels.bumpChannel(accountB.accountAddr, SECRET_2);
    // then fund channel
    _helperFundMultiAB(amount1, amount2);
  }

  /**
   * @dev With a closed channel:
   */
  function _helperWithAClosedChannel(uint256 amount1, uint256 amount2) public {
    // make channel A->B open
    vm.prank(accountB.accountAddr);
    hoprChannels.bumpChannel(accountA.accountAddr, SECRET_2);
    // fund channels
    _helperFundMultiAB(amount1, amount2);

    // accountA initiate channel closure
    vm.startPrank(accountA.accountAddr);
    // initiate channel closure first
    hoprChannels.initiateChannelClosure(accountB.accountAddr);
    // increase enough time for channel closure;
    vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);
    // finalize channel closure
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, amount1), // provide specific
      abi.encode(true)
    );
    hoprChannels.finalizeChannelClosure(accountB.accountAddr);
    vm.stopPrank();
  }

  /**
   * @dev With a reopened channel:
   */
  function _helperWithAReopenedChannel(
    uint256 amount1,
    uint256 amount2,
    uint256 reopenAmount1,
    uint256 reopenAmount2
  ) public {
    // make channel A->B and B=>A open
    _helperOpenBidirectionalChannels(amount1, amount2);

    // accountA initiate channel closure
    vm.startPrank(accountA.accountAddr);
    // initiate channel closure first
    hoprChannels.initiateChannelClosure(accountB.accountAddr);
    // increase enough time for channel closure;
    vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);
    // finalize channel closure
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, amount1), // provide specific
      abi.encode(true)
    );
    hoprChannels.finalizeChannelClosure(accountB.accountAddr);
    vm.stopPrank();

    // fund channel again
    vm.prank(address(1));
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature(
        'transferFrom(address,address,uint256)',
        address(1),
        address(hoprChannels),
        reopenAmount1 + reopenAmount2
      ),
      abi.encode(true)
    );
    // succeed in reopening a channel
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, reopenAmount1, reopenAmount2);
    vm.clearMockedCalls();
  }
}
