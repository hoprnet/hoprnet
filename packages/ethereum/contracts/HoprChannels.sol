// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/utils/introspection/IERC1820Registry.sol";
import "@openzeppelin/contracts/utils/introspection/ERC1820Implementer.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract HoprChannels is IERC777Recipient, ERC1820Implementer {
    using SafeMath for uint256;
    using SafeERC20 for IERC20;

    // required by ERC1820 spec
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    // required by ERC777 spec
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");
    // used by {tokensReceived} to distinguish which function to call after tokens are sent
    uint256 public FUND_CHANNEL_MULTI_SIZE = abi.encode(address(0), address(0), uint256(0), uint256(0)).length;

    /**
     * @dev Possible channel statuses.
     */
    enum ChannelStatus { CLOSED, OPEN, PENDING_TO_CLOSE }

    /**
     * @dev A channel struct, used to represent a channel's state
     */
    struct Channel {
        uint256 partyABalance;
        uint256 partyBBalance;

        bytes32 partyACommitment;
        bytes32 partyBCommitment;
        uint256 partyATicketEpoch;
        uint256 partyBTicketEpoch;
        uint256 partyATicketIndex;
        uint256 partyBTicketIndex;

        ChannelStatus status;
        uint channelEpoch; 

        // the time when the channel can be closed by either party
        // overloads at year >2105
        uint32 closureTime;

        // channel closure was initiated by party A
        bool closureByPartyA;
    }

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

    event Announcement(
        address indexed account,
        bytes multiaddr
    );

    event ChannelUpdate(
        address indexed partyA,
        address indexed partyB,
        Channel newState
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
     * @dev Announces msg.sender's multiaddress.
     * Confirmation should be done off-chain.
     * @param multiaddr the multiaddress
     */
    function announce(bytes calldata multiaddr) external {
        emit Announcement(msg.sender, multiaddr);
    }

    /**
     * @dev Funds a channel, in both directions,
     * then emits {ChannelUpdate} event.
     * @param account1 the address of account1
     * @param account2 the address of account2
     * @param amount1 amount to fund account1
     * @param amount2 amount to fund account2
     */
    function fundChannelMulti(
        address account1,
        address account2,
        uint256 amount1,
        uint256 amount2
    ) external {
        token.safeTransferFrom(msg.sender, address(this), amount1.add(amount2));
        _fundChannel(
            account1,
            account2,
            amount1,
            amount2
        );
    }

    function redeemTicket(
        address counterparty,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        uint256 ticketIndex,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        uint256 winProb,
        bytes memory signature
    ) external {
        _redeemTicket(
            msg.sender,
            counterparty,
            nextCommitment,
            ticketEpoch,
            ticketIndex,
            proofOfRelaySecret,
            amount,
            winProb,
            signature
        );
    }

    /**
     * @dev Initialize channel closure, updates channel'r
     * closure time, when the cool-off period is over,
     * user may finalize closure, then emits
     * {ChannelUpdate} event.
     * @param counterparty the address of the counterparty
     */
    function initiateChannelClosure(
        address counterparty
    ) external {
        _initiateChannelClosure(msg.sender, counterparty);
    }

    /**
     * @dev Finalize channel closure, if cool-off period
     * is over it will close the channel and transfer funds
     * to the parties involved, then emits
     * {ChannelUpdate} event.
     * @param counterparty the address of the counterparty
     */
    function finalizeChannelClosure(
        address counterparty
    ) external {
        _finalizeChannelClosure(
            msg.sender,
            counterparty
        );
    }

    /**
    * @dev Request a channelIteration bump, so we can make a new set of
    * commitments
    * @param counterparty the address of the counterparty
    * @param newCommitment, a secret derived from this new commitment
    */
    function bumpChannel(
      address counterparty,
      bytes32 newCommitment
    ) external {
        require(msg.sender != address(0), "sender must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");
        require(msg.sender != counterparty, "accountA and accountB must not be the same");

        (,,, Channel storage channel) = _getChannel(
            msg.sender,
            counterparty
        );

        if (_isPartyA(msg.sender, counterparty)){
          channel.partyACommitment = newCommitment;
          channel.partyATicketEpoch = channel.partyATicketEpoch.add(1);
        } else {
          channel.partyBCommitment = newCommitment;
          channel.partyATicketEpoch = channel.partyBTicketEpoch.add(1);
        }
    }

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
        require(msg.sender == address(token), "caller must be HoprToken");

        if (
            operator == address(this) || // must not be triggered by HoprChannels
            from == address(0) // ignore 'mint'
        ) {
            return;
        }

        // must be one of our supported functions
        require(
            userData.length == FUND_CHANNEL_MULTI_SIZE,
            "userData must match one of our supported functions"
        );

        address account1;
        address account2;
        uint256 amount1;
        uint256 amount2;

        (account1, account2, amount1, amount2) = abi.decode(userData, (address, address, uint256, uint256));
        require(amount == amount1.add(amount2), "amount sent must be equal to amount specified");

        //require(from == account1 || from == account2, "funder must be either account1 or account2");
        _fundChannel(account1, account2, amount1, amount2);
    }

    // internal code

    /**
     * @dev Funds a channel, then emits
     * {ChannelUpdate} event.
     * @param account1 the address of account1
     * @param account2 the address of account2
     * @param amount1 amount to fund account1
     * @param amount2 amount to fund account2
     */
    function _fundChannel(
        address account1,
        address account2,
        uint256 amount1,
        uint256 amount2
    ) internal {
        require(account1 != account2, "accountA and accountB must not be the same");
        require(account1 != address(0), "accountA must not be empty");
        require(account2 != address(0), "accountB must not be empty");
        require(amount1 > 0 || amount2 > 0, "amountA or amountB must be greater than 0");

        address partyA;
        address partyB;
        uint256 amountA;
        uint256 amountB;
        
        if (_isPartyA(account1, account2)){
          partyA = account1;
          partyB = account2;
          amountA = amount1;
          amountB = amount2;
        } else {
          partyA = account2;
          partyB = account1;
          amountA = amount2;
          amountB = amount1;
        }
        (,,, Channel storage channel) = _getChannel(partyA, partyB);

        require(channel.status != ChannelStatus.PENDING_TO_CLOSE, "Cannot fund a closing channel");
        
        if (channel.status == ChannelStatus.CLOSED) {
          // We are reopening the channel
          channel.channelEpoch = channel.channelEpoch.add(1);
          channel.status = ChannelStatus.OPEN;
          channel.partyATicketIndex = 0;
          channel.partyBTicketIndex = 0;
        }

        channel.partyABalance = channel.partyABalance.add(amountA);
        channel.partyBBalance = channel.partyBBalance.add(amountB);
        emit ChannelUpdate(partyA, partyB, channel);
    }

    /**
     * @dev Initialize channel closure, updates channel's
     * closure time, when the cool-off period is over,
     * user may finalize closure, then emits
     * {ChannelUpdate} event.
     * @param initiator the address of the initiator
     * @param counterparty the address of the counterparty
     */
    function _initiateChannelClosure(
        address initiator,
        address counterparty
    ) internal {
        require(initiator != counterparty, "initiator and counterparty must not be the same");
        require(initiator != address(0), "initiator must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");

        (,,, Channel storage channel) = _getChannel(initiator, counterparty);
        require(channel.status == ChannelStatus.OPEN, "channel must be open");

        // @TODO: check with team, do we need SafeMath check here?
        channel.closureTime = _currentBlockTimestamp() + secsClosure;
        channel.status = ChannelStatus.PENDING_TO_CLOSE;

        bool isPartyA = _isPartyA(initiator, counterparty);
        if (isPartyA) {
            channel.closureByPartyA = true;
        }

        emit ChannelUpdate(initiator, counterparty, channel);
    }

    /**
     * @dev Finalize channel closure, if cool-off period
     * is over it will close the channel and transfer funds
     * to the parties involved, then emits
     * {ChannelUpdate} event.
     * @param initiator the address of the initiator
     * @param counterparty the address of the counterparty
     */
    function _finalizeChannelClosure(
        address initiator,
        address counterparty
    ) internal {
        require(address(token) != address(0), "token must not be empty");
        require(initiator != counterparty, "initiator and counterparty must not be the same");
        require(initiator != address(0), "initiator must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");

        (address partyA, address partyB,, Channel storage channel) = _getChannel(initiator, counterparty);
        require(channel.status == ChannelStatus.PENDING_TO_CLOSE, "channel must be pending to close");

        if (
            channel.closureByPartyA && (initiator == partyA) ||
            !channel.closureByPartyA && (initiator == partyB)
        ) {
            require(channel.closureTime < _currentBlockTimestamp(), "closureTime must be before now");
        }

        // settle balances
        if (channel.partyABalance > 0) {
            token.transfer(partyA, channel.partyABalance);
        }
        if (channel.partyBBalance > 0) {
            token.transfer(partyB, channel.partyBBalance);
        }

        delete channel.partyABalance; // channel.partyABalance = 0
        delete channel.partyBBalance; 
        delete channel.closureTime; // channel.closureTime = 0
        delete channel.closureByPartyA; // channel.closureByPartyA = false
        channel.status = ChannelStatus.CLOSED;

        emit ChannelUpdate(initiator, counterparty, channel);
    }

    /**
     * @param account1 the address of accountA
     * @param account2 the address of accountB
     * @return a tuple of partyA, partyB, channelId, channel
     */
    function _getChannel(address account1, address account2)
        internal
        view
        returns (
            address,
            address,
            bytes32,
            Channel storage
        )
    {
        (address partyA, address partyB) = _sortAddresses(account1, account2);
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
        return keccak256(abi.encodePacked(partyA, partyB));
    }

    /**
     * Parties are ordered - find the lower one.
     * @param query the address of which we are asking 'is this party A'
     * @param other the other address 
     * @return query is partyA 
     */
    function _isPartyA(address query, address other) internal pure returns (bool) {
        return uint160(query) < uint160(other);
    }

    /**
     * @param accountA the address of accountA
     * @param accountB the address of accountB
     * @return a tuple representing partyA and partyB
     */
    function _sortAddresses(address accountA, address accountB) internal pure returns (address, address) {
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
        // solhint-disable-next-line
        return uint32(block.timestamp % 2 ** 32);
    }

    /**
     * @dev Redeem a ticket
     * @param redeemer the redeemer address
     * @param counterparty the counterparty address
     * @param nextCommitment the commitment that hashes to the redeemers previous commitment
     * @param proofOfRelaySecret the proof of relay secret
     * @param winProb the winning probability of the ticket
     * @param amount the amount in the ticket
     * @param signature signature
     */
    function _redeemTicket(
        address redeemer,
        address counterparty,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        uint256 ticketIndex,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        uint256 winProb,
        bytes memory signature
    ) internal {
        require(redeemer != address(0), "redeemer must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");
        require(nextCommitment != bytes32(0), "nextCommitment must not be empty");
        require(amount != uint256(0), "amount must not be empty");
        (,,, Channel storage channel) = _getChannel(
            redeemer,
            counterparty
        );
        
        uint256 prevTicketEpoch;
        if (_isPartyA(redeemer, counterparty)) {
          require(channel.partyACommitment == keccak256(abi.encodePacked(nextCommitment)), "commitment must be hash of next commitment");
          require(channel.partyATicketEpoch == ticketEpoch, "ticket epoch must match");
          require(channel.partyATicketIndex < ticketIndex, "redemptions must be in order");
          prevTicketEpoch = channel.partyATicketEpoch;
        } else {
          require(channel.partyBCommitment == keccak256(abi.encodePacked(nextCommitment)), "commitment must be hash of next commitment");
          require(channel.partyBTicketEpoch == ticketEpoch, "ticket epoch must match");
          require(channel.partyBTicketIndex < ticketIndex, "redemptions must be in order");
          prevTicketEpoch = channel.partyBTicketEpoch;
        }
        require(channel.status != ChannelStatus.CLOSED, "channel must be open or pending to close");

        bytes32 ticketHash = ECDSA.toEthSignedMessageHash(
            keccak256(
              _getEncodedTicket(
                  redeemer,
                  prevTicketEpoch,
                  proofOfRelaySecret,
                  channel.channelEpoch,
                  amount,
                  ticketIndex,
                  winProb
              )
            )
        );

        require(ECDSA.recover(ticketHash, signature) == counterparty, "signer must match the counterparty");
        require(
            uint256(_getTicketLuck(
                ticketHash,
                nextCommitment,
                winProb
            )) <= winProb,
            "ticket must be a win"
        );

        if (_isPartyA(redeemer, counterparty)) {
            channel.partyACommitment = nextCommitment;
            channel.partyABalance = channel.partyABalance.add(amount);
            channel.partyBBalance = channel.partyBBalance.sub(amount);
            channel.partyATicketEpoch = channel.partyATicketEpoch.add(1);
            channel.partyATicketIndex = ticketIndex;
            emit ChannelUpdate(redeemer, counterparty, channel);
        } else {
            channel.partyABalance = channel.partyABalance.sub(amount);
            channel.partyBBalance = channel.partyBBalance.add(amount);
            channel.partyBCommitment = nextCommitment;
            channel.partyBTicketEpoch = channel.partyBTicketEpoch.add(1);
            channel.partyBTicketIndex = ticketIndex;
            emit ChannelUpdate(counterparty, redeemer, channel);
        }
    }

    /**
     * Uses the response to recompute the challenge. This is done
     * by multiplying the base point of the curve with the given response.
     * Due to the lack of embedded ECMUL functionality in the current
     * version of the EVM, this is done by misusing the `ecrecover` 
     * functionality. `ecrecover` performs the point multiplication and 
     * converts the output to an Ethereum address (sliced hash of the product
     * of base point and scalar).
     * See https://ethresear.ch/t/you-can-kinda-abuse-ecrecover-to-do-ecmul-in-secp256k1-today/2384
     * @param response response that is used to recompute the challenge
     */
    function computeChallenge(bytes32 response) public pure returns (address)  {
        // Field order of the base field
        uint256 FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;

        // x-coordinate of the base point
        uint256 gx = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;
        // y-coordinate of base-point is even, so v is 27
        uint8 gv = 27;

        address signer = ecrecover(0, gv, bytes32(gx), bytes32(mulmod(uint256(response), gx, FIELD_ORDER)));

        return signer;
    }
    /**
     * @dev Encode ticket data
     * @return bytes
     */
    function _getEncodedTicket(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        uint256 ticketIndex,
        uint256 winProb
    ) internal pure returns (bytes memory) {
        address challenge = computeChallenge(proofOfRelaySecret);

        return abi.encodePacked(
            recipient,
            challenge,
            recipientCounter,
            amount,
            winProb,
            ticketIndex,
            channelIteration
        );
    }
    
    /**
     * @dev Get the ticket's "luck" by
     * hashing provided values.
     * @return luck
     */
    function _getTicketLuck(
        bytes32 ticketHash,
        bytes32 nextCommitment,
        uint256 winProb
    ) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(ticketHash, nextCommitment, winProb));
    }
}
