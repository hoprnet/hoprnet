// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import '@openzeppelin/contracts/math/SafeMath.sol';
import '@openzeppelin/contracts/introspection/IERC1820Registry.sol';
import '@openzeppelin/contracts/introspection/ERC1820Implementer.sol';
import '@openzeppelin/contracts/token/ERC20/IERC20.sol';
import '@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol';
import '@openzeppelin/contracts/token/ERC20/SafeERC20.sol';
import './utils/ECDSA.sol';
import './utils/SafeUint24.sol';
import './utils/SafeUint32.sol';
import './utils/SafeUint96.sol';

pragma experimental ABIEncoderV2;

contract HoprChannels is IERC777Recipient, ERC1820Implementer {
  using SafeUint24 for uint24;
  using SafeUint32 for uint32;
  using SafeUint96 for uint96;

  using SafeERC20 for IERC20;

  // required by ERC1820 spec
  IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
  // required by ERC777 spec
  bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256('ERC777TokensRecipient');
  // used by {tokensReceived} to distinguish which function to call after tokens are sent
  uint256 public FUND_CHANNEL_SIZE = abi.encode(false, address(0), address(0)).length;
  // used by {tokensReceived} to distinguish which function to call after tokens are sent
  uint256 public FUND_CHANNEL_MULTI_SIZE = abi.encode(false, address(0), address(0), uint256(0), uint256(0)).length;

  string private constant TICKET_LENGTH = '187';

  uint256 private constant MAX_TIMESTAMP = (1 << 32) - 1;

  uint24 private constant STATE_COUNT = 8;

  uint24 private constant STATE_MASK = (1 << 24) - 8;

  uint256 private constant STATE_MAX = (1 << 24) - 16;

  /**
   * @dev An account struct, used to represent an account's state
   */
  struct Account {
    bytes27 secret; // account's hashed secret
    uint32 counter; // increases everytime 'secret' is changed
    uint8 oddY; // sign bit of public key
    uint256 pubKey; // public key
  }

  struct Ticket {
    address counterparty;
    uint96 amount;
    bytes32 proofOfRelaySecret;
    bytes32 winProb;
    bytes27 secretPreImage;
    uint32 nonce;
    uint8 v;
    bytes32 r;
    bytes32 s;
  }

  /**
   * @dev Possible channel statuses.
   * We find out the channel's status by
   * using {_getChannelStatus}.
   */
  enum ChannelStatus {CLOSED, OPEN, PENDING_TO_CLOSE}

  /**
   * @dev A channel struct, used to represent a channel's state
   */
  struct Channel {
    // total tokens in deposit
    uint96 deposit;
    // tokens that are claimable by partyA
    uint96 partyABalance;
    // overflow after ~ 2 PiB traffic per channel
    uint32 nonceA;
    // overflow after ~ 2 PiB traffic per channel
    uint32 nonceB;
    // the time when the channel can be closed by either party
    // overloads at year >2105
    uint32 closureTime;
    // status of the channel
    // overloads at >16777215
    uint24 status;
    // channel closure was initiated by party A
    bool closureByPartyA;
  }

  /**
   * @dev Stored accounts keyed by their address
   */
  mapping(address => Account) public accounts;

  /**
   * @dev Stored channels keyed by their channel ids
   */
  mapping(bytes32 => Channel) public channels;

  /**
   * @dev HoprToken, the token that will be used to settle payments
   */
  IERC20 public token;

  /**
   * @dev Seconds it takes until we can finalize channel closure once,
   * channel closure has been initialized.
   */
  uint32 public secsClosure;

  event AccountInitializedEven(uint256 indexed pubKeyFirstHalf);
  event AccountInitializedOdd(uint256 indexed pubKeyFirstHalf);

  event AccountSecretUpdated(address indexed account, bytes27 secret);

  event ChannelFunded(address indexed funder, address indexed counterparty, uint96 deposit, uint96 partyABalance);

  // Consumed by HOPR client and used to encode public keys
  event ChannelOpenedEvenEven(uint256 indexed opener, uint256 indexed counterparty);
  event ChannelOpenedEvenOdd(uint256 indexed opener, uint256 indexed counterparty);
  event ChannelOpenedOddEven(uint256 indexed opener, uint256 indexed counterparty);
  event ChannelOpenedOddOdd(uint256 indexed opener, uint256 indexed counterparty);

  // Consumed by HOPR client and used to encode public keys
  event ChannelPendingToCloseEvenEven(uint256 indexed initiator, uint256 indexed counterparty, uint32 closureTime);
  event ChannelPendingToCloseEvenOdd(uint256 indexed initiator, uint256 indexed counterparty, uint32 closureTime);
  event ChannelPendingToCloseOddEven(uint256 indexed initiator, uint256 indexed counterparty, uint32 closureTime);
  event ChannelPendingToCloseOddOdd(uint256 indexed initiator, uint256 indexed counterparty, uint32 closureTime);

  event ChannelClosed(
    address indexed initiator,
    address indexed counterparty,
    uint96 partyAAmount,
    uint96 partyBAmount
  );

  /**
   * @param _token HoprToken address
   * @param _secsClosure seconds until a channel can be closed
   */
  constructor(address _token, uint32 _secsClosure) {
    token = IERC20(_token);
    secsClosure = _secsClosure;
    _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
  }

  /**
   * @dev Initializes an account,
   * stores it's public key, secret and counter,
   * then emits {AccountInitialized} and {AccountSecretUpdated} events.
   * @param secret account's secret
   * @param pubKeyFirstHalf first half of the public key
   * @param pubKeySecondHalf second half of the public key
   */
  function initializeAccount(
    uint256 pubKeyFirstHalf,
    uint256 pubKeySecondHalf,
    bytes27 secret
  ) external {
    require(pubKeyFirstHalf != uint256(0), 'pubKeyFirstHalf must not be empty');
    require(pubKeySecondHalf != uint256(0), 'pubKeySecondHalf must not be empty');
    require(secret != bytes27(0), 'secret must not be empty');

    require(
      ECDSA.pubKeyToEthereumAddress(pubKeyFirstHalf, pubKeySecondHalf) == msg.sender,
      'public key does not match account'
    );

    Account storage accountData = accounts[msg.sender];

    require(
      accountData.secret == 0 && accountData.counter == 0 && accountData.oddY == 0 && accountData.pubKey == 0,
      'Account must not be set'
    );

    uint8 oddY = uint8(pubKeySecondHalf & 1);

    accountData.secret = secret;
    accountData.counter = 1;
    accountData.oddY = oddY;
    accountData.pubKey = pubKeyFirstHalf;

    if (oddY == 1) {
      emit AccountInitializedOdd(pubKeyFirstHalf);
    } else {
      emit AccountInitializedEven(pubKeyFirstHalf);
    }
  }

  /**
   * @dev Updates account's secret and counter,
   * then emits {AccountSecretUpdated} event.
   * @param secret account's secret
   */
  function updateAccount(bytes27 secret) external {
    require(secret != bytes27(0), 'secret must not be empty');

    Account storage accountData = accounts[msg.sender];

    require(
      accountData.secret != 0 || accountData.counter != 0 || accountData.oddY != 0 || accountData.pubKey != 0,
      'Account must be set'
    );
    require(secret != accountData.secret, 'secret must not be the same as before');

    accountData.secret = secret;

    accountData.counter = accountData.counter.add(1);

    emit AccountSecretUpdated(msg.sender, secret);
  }

  /**
   * @dev Funds a channel in one direction,
   * then emits {ChannelFunded} event.
   * @param counterparty the address of the counterparty
   * @param amount amount to fund
   */
  function fundChannel(address counterparty, uint96 amount) external {
    token.safeTransferFrom(msg.sender, address(this), amount);

    _fundChannel(msg.sender, msg.sender, counterparty, amount, 0);
  }

  /**
   * @dev Funds a channel in one direction for a third-party,
   * then emits {ChannelFunded} event.
   * @param account the address of the recipient
   * @param counterparty the address of the counterparty
   * @param amount amount to fund
   */
  function fundChannel(
    address account,
    uint96 amount,
    address counterparty
  ) external {
    token.safeTransferFrom(msg.sender, address(this), amount);

    _fundChannel(msg.sender, account, counterparty, amount, 0);
  }

  /**
   * @dev Funds a channel, in both directions for two third-parties,
   * then emits {ChannelFunded} event.
   * @param counterparty the address of accountB
   * @param amountA amount to fund accountA
   * @param amountB amount to fund accountB
   */
  function fundChannelBothDirections(
    address counterparty,
    uint96 amountA,
    uint96 amountB
  ) external {
    token.safeTransferFrom(msg.sender, address(this), uint256(amountA.add(amountB)));

    _fundChannel(msg.sender, msg.sender, counterparty, amountA, amountB);
  }

  /**
   * @dev Funds a channel, in both directions,
   * then emits {ChannelFunded} event.
   * @param accountA the address of accountA
   * @param accountB the address of accountB
   * @param amountA amount to fund accountA
   * @param amountB amount to fund accountB
   */
  function fundChannelBothDirections(
    address accountA,
    uint96 amountA,
    address accountB,
    uint96 amountB
  ) external {
    token.safeTransferFrom(msg.sender, address(this), uint256(amountA.add(amountB)));

    _fundChannel(msg.sender, accountA, accountB, amountA, amountB);
  }

  /**
   * @dev Opens a channel, then emits
   * {ChannelOpened} event.
   * @param counterparty the address of the counterparty
   */
  function openChannel(address counterparty) external {
    _openChannel(msg.sender, counterparty);
  }

  /**
   * @dev Fund channel for two third-parties
   * and then open it, then emits
   * {ChannelFunded} and {ChannelOpened} events.
   * @param accountA the address of accountA
   * @param accountB the address of accountB
   * @param amountA amount to fund accountA
   * @param amountB amount to fund accountB
   */
  function fundAndOpenChannel(
    address accountA,
    uint96 amountA,
    address accountB,
    uint96 amountB
  ) external {
    address opener = msg.sender;
    require(opener == accountA || opener == accountB, 'opener must be accountA or accountB');

    token.safeTransferFrom(msg.sender, address(this), amountA.add(amountB));

    address counterparty;
    if (opener == accountA) {
      counterparty = accountB;
    } else {
      counterparty = accountA;
    }

    _fundChannel(opener, accountA, accountB, amountA, amountB);
    _openChannel(opener, counterparty);
  }

  function redeemTicketDry(Ticket calldata ticket) external {
    _verifyAndInvalidateTicket(ticket);
  }

  function redeemTicket(Ticket calldata ticket) external {
    (Channel storage channel, bytes32 ticketHash) = _verifyAndInvalidateTicket(ticket);

    require(
      uint256(
        keccak256(abi.encodePacked(ticketHash, ticket.secretPreImage, ticket.proofOfRelaySecret, ticket.winProb))
      ) <= uint256(ticket.winProb),
      'ticket must be a win'
    );

    if (_isPartyA(msg.sender, ticket.counterparty)) {
      channel.partyABalance = channel.partyABalance.add(ticket.amount);
    } else {
      channel.partyABalance = channel.partyABalance.sub(ticket.amount);
    }
  }

  /**
   * @dev Initialize channel closure, updates channel's
   * closure time, when the cool-off period is over,
   * user may finalize closure, then emits
   * {ChannelPendingToClose} event.
   * @param counterparty the address of the counterparty
   */
  function initiateChannelClosure(address counterparty) external {
    _initiateChannelClosure(msg.sender, counterparty);
  }

  /**
   * @dev Finalize channel closure, if cool-off period
   * is over it will close the channel and transfer funds
   * to the parties involved, then emits
   * {ChannelClosed} event.
   * @param counterparty the address of the counterparty
   */
  function finalizeChannelClosure(address counterparty) external {
    _finalizeChannelClosure(msg.sender, counterparty);
  }

  // @TODO: check with team, is this function too complex?
  // @TODO: should we support account init?
  /**
   * A hook triggered when HOPR tokens are send to this contract.
   *
   * @param operator address operator requesting the transfer
   * @param from address token holder address
   * @param to address recipient address
   * @param amount uint256 amount of tokens to transfer
   * @param userData bytes extra information provided by the token holder (if any)
   * @param operatorData bytes extra information provided by the operator (if any)
   */
  function tokensReceived(
    address operator,
    address from,
    // solhint-disable-next-line no-unused-vars
    address to,
    uint256 amount,
    bytes calldata userData,
    // solhint-disable-next-line no-unused-vars
    bytes calldata operatorData
  ) external override {
    require(msg.sender == address(token), 'caller must be HoprToken');

    if (
      operator == address(this) || // must not be triggered by HoprChannels
      from == address(0) // ignore 'mint'
    ) {
      return;
    }

    // must be one of our supported functions
    require(
      userData.length == FUND_CHANNEL_SIZE || userData.length == FUND_CHANNEL_MULTI_SIZE,
      'userData must match one of our supported functions'
    );

    bool shouldOpen;
    address accountA;
    uint96 amountA;

    address accountB;
    uint96 amountB;

    uint256 _amountA;
    uint256 _amountB;

    if (userData.length == FUND_CHANNEL_SIZE) {
      (shouldOpen, accountA, accountB) = abi.decode(userData, (bool, address, address));
      _amountA = uint96(amount);
    } else {
      (shouldOpen, accountA, accountB, _amountA, _amountB) = abi.decode(
        userData,
        (bool, address, address, uint256, uint256)
      );
      // @TODO find bettet solution
      amountA = uint96(_amountA);
      amountB = uint96(_amountB);

      require(uint96(amount) == uint96(amountA).add(amountB), 'amount sent must be equal to amount specified');
    }

    _fundChannel(from, accountA, accountB, amountA, amountB);

    if (shouldOpen) {
      require(from == accountA || from == accountB, 'funder must be either accountA or accountB');
      _openChannel(accountA, accountB);
    }
  }

  // internal code

  /**
   * @dev Funds a channel, then emits
   * {ChannelFunded} event.
   * @param funder the address of the funder
   * @param accountA the address of accountA
   * @param accountB the address of accountB
   * @param amountA amount to fund accountA
   * @param amountB amount to fund accountB
   */
  function _fundChannel(
    address funder,
    address accountA,
    address accountB,
    uint96 amountA,
    uint96 amountB
  ) internal {
    require(funder != address(0), 'funder must not be empty');
    require(accountA != accountB, 'accountA and accountB must not be the same');
    require(accountA != address(0), 'accountA must not be empty');
    require(accountB != address(0), 'accountB must not be empty');
    require(amountA > 0 || amountB > 0, 'amountA or amountB must be greater than 0');

    Channel storage channel = channels[_getChannelId(accountA, accountB)];

    channel.deposit = channel.deposit.add(amountA).add(amountB);

    if (_isPartyA(accountA, accountB)) {
      channel.partyABalance = channel.partyABalance.add(amountA);
    }

    emit ChannelFunded(accountA, accountB, channel.deposit, channel.partyABalance);
  }

  /**
   * @dev Opens a channel, then emits
   * {ChannelOpened} event.
   * @param opener the address of the opener
   * @param counterparty the address of the counterparty
   */
  function _openChannel(address opener, address counterparty) internal {
    require(opener != counterparty, 'opener and counterparty must not be the same');
    require(opener != address(0), 'opener must not be empty');
    require(counterparty != address(0), 'counterparty must not be empty');

    Channel storage channel = channels[_getChannelId(opener, counterparty)];
    require(channel.deposit > 0, 'channel must be funded');

    ChannelStatus channelStatus = _getChannelStatus(channel.status);
    require(channelStatus == ChannelStatus.CLOSED, 'channel must be closed in order to open');

    channel.status = _setChannelStatus(channel.status, ChannelStatus.OPEN);

    Account storage openerAccount = accounts[opener];
    Account storage counterpartyAccount = accounts[counterparty];

    if (openerAccount.oddY == 1) {
      if (counterpartyAccount.oddY == 1) {
        emit ChannelOpenedOddOdd(openerAccount.pubKey, counterpartyAccount.pubKey);
      } else {
        emit ChannelOpenedOddEven(openerAccount.pubKey, counterpartyAccount.pubKey);
      }
    } else {
      if (counterpartyAccount.oddY == 1) {
        emit ChannelOpenedEvenOdd(openerAccount.pubKey, counterpartyAccount.pubKey);
      } else {
        emit ChannelOpenedEvenEven(openerAccount.pubKey, counterpartyAccount.pubKey);
      }
    }
  }

  /**
   * @dev Initialize channel closure, updates channel's
   * closure time, when the cool-off period is over,
   * user may finalize closure, then emits
   * {ChannelPendingToClose} event.
   * @param initiator the address of the initiator
   * @param counterparty the address of the counterparty
   */
  function _initiateChannelClosure(address initiator, address counterparty) internal {
    require(initiator != counterparty, 'initiator and counterparty must not be the same');
    require(initiator != address(0), 'initiator must not be empty');
    require(counterparty != address(0), 'counterparty must not be empty');

    Channel storage channel = channels[_getChannelId(initiator, counterparty)];
    require(_getChannelStatus(channel.status) == ChannelStatus.OPEN, 'channel must be open');

    // @TODO: check with team, do we need SafeMath check here?
    channel.closureTime = _currentBlockTimestamp() + secsClosure;
    channel.status = _setChannelStatus(channel.status, ChannelStatus.PENDING_TO_CLOSE);

    bool isPartyA = _isPartyA(initiator, counterparty);
    if (isPartyA) {
      channel.closureByPartyA = true;
    }

    Account storage openerAccount = accounts[initiator];
    Account storage counterpartyAccount = accounts[counterparty];

    if (openerAccount.oddY == 1) {
      if (counterpartyAccount.oddY == 1) {
        emit ChannelPendingToCloseOddOdd(openerAccount.pubKey, counterpartyAccount.pubKey, channel.closureTime);
      } else {
        emit ChannelPendingToCloseOddEven(openerAccount.pubKey, counterpartyAccount.pubKey, channel.closureTime);
      }
    } else {
      if (counterpartyAccount.oddY == 1) {
        emit ChannelPendingToCloseEvenOdd(openerAccount.pubKey, counterpartyAccount.pubKey, channel.closureTime);
      } else {
        emit ChannelPendingToCloseEvenEven(openerAccount.pubKey, counterpartyAccount.pubKey, channel.closureTime);
      }
    }
  }

  /**
   * @dev Finalize channel closure, if cool-off period
   * is over it will close the channel and transfer funds
   * to the parties involved, then emits
   * {ChannelClosed} event.
   * @param initiator the address of the initiator
   * @param counterparty the address of the counterparty
   */
  function _finalizeChannelClosure(address initiator, address counterparty) internal {
    require(address(token) != address(0), 'token must not be empty');
    require(initiator != counterparty, 'initiator and counterparty must not be the same');
    require(initiator != address(0), 'initiator must not be empty');
    require(counterparty != address(0), 'counterparty must not be empty');

    (address partyA, address partyB) = _getParties(initiator, counterparty);
    Channel storage channel = channels[_getChannelId(partyA, partyB)];

    require(_getChannelStatus(channel.status) == ChannelStatus.PENDING_TO_CLOSE, 'channel must be pending to close');

    if ((channel.closureByPartyA && (initiator == partyA)) || (!channel.closureByPartyA && (initiator == partyB))) {
      // @TODO: this does not work
      require(channel.closureTime < _currentBlockTimestamp(), 'closureTime must be before now');
    }

    uint96 partyAAmount = channel.partyABalance;
    uint96 partyBAmount = channel.deposit.sub(channel.partyABalance);

    // settle balances
    if (partyAAmount > 0) {
      token.transfer(partyA, partyAAmount);
    }
    if (partyBAmount > 0) {
      token.transfer(partyB, partyBAmount);
    }

    // The state counter indicates the recycling generation and ensures that both parties are using the correct generation.
    // Increase state counter so that we can re-use the same channel after it has been closed.
    channel.status = channel.status.add(8);
    delete channel.deposit; // channel.deposit = 0
    delete channel.partyABalance; // channel.partyABalance = 0
    delete channel.closureTime; // channel.closureTime = 0
    delete channel.closureByPartyA; // channel.closureByPartyA = false

    emit ChannelClosed(initiator, counterparty, partyAAmount, partyBAmount);
  }

  /**
   * @param accountA the address of accountA
   * @param accountB the address of accountB
   * @return a tuple of partyA, partyB, channelId, channel
   */
  function _getChannel(address accountA, address accountB)
    internal
    view
    returns (
      address,
      address,
      bytes32,
      Channel storage
    )
  {
    (address partyA, address partyB) = _getParties(accountA, accountB);
    bytes32 channelId = _getChannelId(partyA, partyB);
    Channel storage channel = channels[channelId];

    return (partyA, partyB, channelId, channel);
  }

  /**
   * @param partyA the address of partyA
   * @param partyB the address of partyB
   * @return the channel id by hashing partyA and partyB
   */
  function _getChannelId(address partyA, address partyB) internal pure returns (bytes32) {
    if (_isPartyA(partyA, partyB)) {
      return keccak256(abi.encodePacked(partyA, partyB));
    } else {
      return keccak256(abi.encodePacked(partyB, partyA));
    }
  }

  /**
   * @param status channel's status
   * @return the channel's status in 'ChannelStatus'
   */
  function _getChannelStatus(uint24 status) internal pure returns (ChannelStatus) {
    return ChannelStatus(status.mod(STATE_COUNT));
  }

  function _setChannelStatus(uint24 status, ChannelStatus newStatus) internal pure returns (uint24) {
    if (newStatus == ChannelStatus.CLOSED) {
      return (status & STATE_MASK).add(STATE_COUNT);
    } else {
      return (status & STATE_MASK).add(uint24(newStatus));
    }
  }

  /**
   * @param status channel's status
   * @return the channel's iteration
   */
  function _getChannelIteration(uint24 status) internal pure returns (uint256) {
    return status.div(STATE_COUNT).add(1);
  }

  /**
   * @param accountA the address of accountA
   * @param accountB the address of accountB
   * @return true if accountA is partyA
   */
  function _isPartyA(address accountA, address accountB) internal pure returns (bool) {
    return uint160(accountA) < uint160(accountB);
  }

  /**
   * @param accountA the address of accountA
   * @param accountB the address of accountB
   * @return a tuple representing partyA and partyB
   */
  function _getParties(address accountA, address accountB) internal pure returns (address, address) {
    if (_isPartyA(accountA, accountB)) {
      return (accountA, accountB);
    } else {
      return (accountB, accountA);
    }
  }

  /**
   * @return the current timestamp
   */
  function _currentBlockTimestamp() internal view returns (uint32) {
    require(block.timestamp <= MAX_TIMESTAMP, 'Timestamp too big. Please use the upgraded smart contract.');

    return uint32(block.timestamp);
  }

  /**
   * @dev Checks the validity of the given ticket and
   * updates the stored values such that the ticket
   * becomes invalid.
   */
  function _verifyAndInvalidateTicket(Ticket memory ticket) internal returns (Channel storage, bytes32) {
    require(ticket.counterparty != address(0), 'counterparty must not be empty');
    require(ticket.secretPreImage != bytes32(0), 'secretPreImage must not be empty');
    require(ticket.proofOfRelaySecret != bytes32(0), 'proofOfRelaySecret must not be empty');
    require(ticket.amount != uint96(0), 'amount must not be empty');
    require(ticket.winProb != bytes32(0), 'winProb must not be empty');
    require(ticket.r != bytes32(0), 'r must not be empty');
    require(ticket.s != bytes32(0), 's must not be empty');
    require(ticket.v != uint8(0), 'v must not be empty');

    Account storage account = accounts[msg.sender];
    require(
      account.secret == bytes27(keccak256(abi.encodePacked('HOPRnet', msg.sender, bytes27(ticket.secretPreImage)))),
      "secretPreImage must be the hash of recipient's secret"
    );

    Channel storage channel = channels[_getChannelId(msg.sender, ticket.counterparty)];
    require(_getChannelStatus(channel.status) != ChannelStatus.CLOSED, 'channel must be open or pending to close');

    bytes32 challenge = keccak256(abi.encodePacked(ticket.proofOfRelaySecret));

    bytes32 ticketHash =
      ECDSA.toEthSignedMessageHash(
        TICKET_LENGTH,
        abi.encodePacked(
          msg.sender,
          challenge,
          account.counter,
          ticket.amount,
          ticket.winProb,
          _getChannelIteration(channel.status)
        )
      );

    require(
      ECDSA.recover(ticketHash, ticket.r, ticket.s, ticket.v) == ticket.counterparty,
      'signer must match the counterparty'
    );

    if (_isPartyA(msg.sender, ticket.counterparty)) {
      require(channel.nonceB < ticket.nonce);
      channel.nonceB = ticket.nonce;
    } else {
      require(channel.nonceA < ticket.nonce);
      channel.nonceA = ticket.nonce;
    }

    account.secret = ticket.secretPreImage;

    return (channel, ticketHash);
  }
}
