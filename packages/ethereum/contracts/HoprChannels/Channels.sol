// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "../utils/console.sol";

contract Channels {
    /**
     * @dev Possible channel statuses.
     * We find out the channel's status by
     * using {_getChannelStatus}.
     */
    enum ChannelStatus { CLOSED, OPEN, PENDING_TO_CLOSE }

    /**
     * @dev A channel struct
     */
    struct Channel {
        // total tokens in deposit
        uint256 deposit;
        // tokens that are claimable by partyA
        uint256 partyABalance;
        // the time when the channel can be closed by either party
        // overloads at year 2106
        uint32 closureTime;
        // status of the channel
        // overloads at >16777215
        uint24 status;
        // channel closure was initiated by party A
        bool closureByPartyA;
    }

    /**
     * @dev Seconds it takes until we can finalize channel closure once,
     * channel closure has been initialized.
     */
    uint32 public secsClosure;

    /**
     * @dev Stored channels keyed by their channel ids
     */
    mapping(bytes32 => Channel) public channels;

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
        uint256 amountA,
        uint256 amountB
    ) internal {
        // require(funder != address(0), "funder must not be empty");
        require(accountA != accountB, "accountA and accountB must not be the same");
        require(accountA != address(0), "accountA must not be empty");
        require(accountB != address(0), "accountB must not be empty");
        require(amountA > 0 || amountB > 0, "amountA or amountB must be greater than 0");

        (,,, Channel storage channel) = _getChannel(accountA, accountB);

        // @TODO: use SafeMath
        channel.deposit += (amountA + amountB);
        if (_isPartyA(accountA, accountB)) {
            channel.partyABalance += amountA;
        }

        emit ChannelFunded(
            accountA,
            accountB,
            funder,
            channel.deposit,
            channel.partyABalance
        );
    }

    /**
     * @dev Opens a channel, then emits
     * {ChannelOpened} event.
     * @param opener the address of the opener
     * @param counterparty the address of the counterparty
     */
    function _openChannel(
        address opener,
        address counterparty
    ) internal {
        require(opener != counterparty, "opener and counterparty must not be the same");
        require(opener != address(0), "opener must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");

        (,,, Channel storage channel) = _getChannel(opener, counterparty);
        require(channel.deposit > 0, "channel must be funded");

        ChannelStatus channelStatus = _getChannelStatus(channel.status);
        require(channelStatus == ChannelStatus.CLOSED, "channel must be closed in order to open");

        channel.status += 1;

        emit ChannelOpened(opener, counterparty);
    }

    /**
     * @dev Initialize channel closure, updates channel's
     * closure time, when the cool-off period is over,
     * user may finalize closure, then emits
     * {ChannelPendingToClose} event.
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
        ChannelStatus channelStatus = _getChannelStatus(channel.status);
        require(
            channelStatus == ChannelStatus.OPEN,
            "channel must be open"
        );

        // TODO: use SafeMath
        channel.closureTime = _currentBlockTimestamp() + secsClosure;
        channel.status += 1;

        bool isPartyA = _isPartyA(initiator, counterparty);
        if (isPartyA) {
            channel.closureByPartyA = true;
        }

        emit ChannelPendingToClose(initiator, counterparty, channel.closureTime);
    }

    /**
     * @dev Finalize channel closure, if cool-off period
     * is over it will close the channel and transfer funds
     * to the parties involved, then emits
     * {ChannelClosed} event.
     * @param token an ERC20 compatible token
     * @param initiator the address of the initiator
     * @param counterparty the address of the counterparty
     */
    function _finalizeChannelClosure(
        IERC20 token,
        address initiator,
        address counterparty
    ) internal {
        require(address(token) != address(0), "token must not be empty");
        require(initiator != counterparty, "initiator and counterparty must not be the same");
        require(initiator != address(0), "initiator must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");

        (address partyA, address partyB,, Channel storage channel) = _getChannel(initiator, counterparty);
        ChannelStatus channelStatus = _getChannelStatus(channel.status);
        require(
            channelStatus == ChannelStatus.PENDING_TO_CLOSE,
            "channel must be pending to close"
        );

        if (
            channel.closureByPartyA && (initiator == partyA) ||
            !channel.closureByPartyA && (initiator == partyB)
        ) {
            require(channel.closureTime < _currentBlockTimestamp(), "closureTime must be before now");
        }

        uint256 partyAAmount = channel.partyABalance;
        // @TODO: add SafeMath
        uint256 partyBAmount = channel.deposit - channel.partyABalance;

        // settle balances
        if (partyAAmount > 0) {
            token.transfer(partyA, partyAAmount);
        }
        if (partyBAmount > 0) {
            token.transfer(partyB, partyBAmount);
        }

        // The state counter indicates the recycling generation and ensures that both parties are using the correct generation.
        // Increase state counter so that we can re-use the same channel after it has been closed.
        channel.status += 8;
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
        return keccak256(abi.encodePacked(partyA, partyB));
    }

    /**
     * @param status channel's status
     * @return the channel's status in 'ChannelStatus'
     */
    function _getChannelStatus(uint24 status) internal pure returns (ChannelStatus) {
        return ChannelStatus(status % 10);
    }

    /**
     * @param status channel's status
     * @return the channel's iteration
     */
    function _getChannelIteration(uint24 status) internal pure returns (uint256) {
        return (status / 10) + 1;
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
        // solhint-disable-next-line
        return uint32(block.timestamp % 2 ** 32);
    }

    event ChannelFunded(
        address indexed accountA,
        address indexed accountB,
        // @TODO: remove this and rely on `msg.sender`
        address funder,
        uint256 deposit,
        uint256 partyABalance
    );

    event ChannelOpened(
        // @TODO: remove this and rely on `msg.sender`
        address indexed opener,
        address indexed counterparty
    );

    event ChannelPendingToClose(
        // @TODO: remove this and rely on `msg.sender`
        address indexed initiator,
        address indexed counterparty,
        uint256 closureTime
    );

    event ChannelClosed(
        // @TODO: remove this and rely on `msg.sender`
        address indexed initiator,
        address indexed counterparty,
        uint256 partyAAmount,
        uint256 partyBAmount
    );
}