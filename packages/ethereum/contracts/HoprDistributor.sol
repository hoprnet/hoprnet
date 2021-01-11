// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.6.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./HoprToken.sol";

contract HoprDistributor is Ownable {
    // helps us create more accurate calculations
    uint32 public constant MULTIPLIER = 10 ** 6;

    // total amount minted
    uint128 public totalMinted = 0;
    // how many tokens will be minted (the sum of all allocations)
    uint128 public totalToBeMinted = 0;

    // time where the contract will consider as starting time
    uint32 public startTime;
    // token which will be used
    HoprToken public token;
    // maximum tokens allowed to be minted
    uint128 public maxMintAmount;

    // A {Schedule} that defined when and how much will be claimed
    // from an {Allocation}
    struct Schedule {
        uint32[] durations;
        uint32[] percents;
    }

    // An {Allocation} represents how much a account can claim, claimed,
    // and when last claim occured
    struct Allocation {
        uint128 amount;
        uint128 claimed;
        uint32 lastClaim;
        bool revoked; // account can no longer claim
    }

    // schedule name -> Schedule
    mapping(string => Schedule) internal schedules;

    // account -> schedule name -> Allocation
    // allows for an account to have more than one type of Schedule
    mapping(address => mapping(string => Allocation)) public allocations;

    /**
     * @param _startTime The timestamp to start counting
     * @param _token The token which we will mint
     */
    constructor(HoprToken _token, uint32 _startTime, uint128 _maxMintAmount) public {
        startTime = _startTime;
        token = _token;
        maxMintAmount = _maxMintAmount;
    }

    /**
     * @param name the schedule name
     * @return the schedule
     */
    function getSchedule(string calldata name) external view returns (uint32[] memory, uint32[] memory) {
        return (
            schedules[name].durations,
            schedules[name].percents
        );
    }

    /**
     * @dev Revokes the ability for an account to claim on the
     * specified schedule.
     * @param account the account to crevoke
     * @param scheduleName the schedule name
     */
    function revokeAccount(
        address account,
        string calldata scheduleName
    ) external onlyOwner {
        Allocation storage allocation = allocations[account][scheduleName];
        require(allocation.amount != 0, "Allocation must exist");

        allocation.revoked = true;
        totalToBeMinted = _subUint128(totalToBeMinted, _subUint128(allocation.amount, allocation.claimed));
    }

    /**
     * @dev Adds a schedule, the schedule must not already exist.
     * Owner is expected to insert values in ascending order,
     * each element in arrays {durations} and {percents} is meant to be
     * related.
     * @param durations the durations for each schedule period in seconds (6 months, 1 year)
     * @param percents the percent of how much can be allocated during that period,
     * instead of using 100 we scale the value up to {MULTIPLIER} so we can have more accurate
     * "percentages".
     */
    function addSchedule(
        uint32[] calldata durations,
        uint32[] calldata percents,
        string calldata name
    ) external onlyOwner {
        require(schedules[name].durations.length == 0, "Schedule must not exist");
        require(durations.length == percents.length, "Durations and percents must have equal length");

        uint32 lastDuration = 0;
        for (uint256 i = 0; i < durations.length; i++) {
            require(lastDuration <= durations[i], "Durations must be added in ascending order");
            lastDuration = durations[i];
            require(percents[i] <= MULTIPLIER, "Percent provided must be smaller or equal to MULTIPLIER");
        }

        schedules[name] = Schedule(durations, percents);

        emit ScheduleAdded(durations, percents, name);
    }

    /**
     * @dev Adds allocations, all allocations will use the schedule specified,
     * schedule must be created before and account must not have an allocation
     * in the specific schedule.
     * @param accounts accounts to create allocations for
     * @param amounts total amount to be allocated
     * @param scheduleName the schedule name
     */
    function addAllocations(
        address[] calldata accounts,
        uint128[] calldata amounts,
        string calldata scheduleName
    ) external onlyOwner {
        require(schedules[scheduleName].durations.length != 0, "Schedule must exist");
        require(accounts.length == amounts.length, "Accounts and amounts must have equal length");

        for (uint256 i = 0; i < accounts.length; i++) {
            require(allocations[accounts[i]][scheduleName].amount == 0, "Allocation must not exist");
            allocations[accounts[i]][scheduleName].amount = amounts[i];
            totalToBeMinted = _addUint128(totalToBeMinted, amounts[i]);
            assert(totalToBeMinted <= maxMintAmount);

            emit AllocationAdded(accounts[i], amounts[i], scheduleName);
        }
    }

    /**
     * @dev Claim tokens by specified a schedule.
     * @param scheduleName the schedule name
     */
    function claim(string calldata scheduleName) external {
        return _claim(msg.sender, scheduleName);
    }

    /**
     * @dev Claim tokens for a specific account by specified a schedule.
     * @param account the account to claim for
     * @param scheduleName the schedule name
     */
    function claimFor(address account, string calldata scheduleName) external {
        return _claim(account, scheduleName);
    }

    /**
     * @param account the account to get claimable for
     * @param scheduleName the schedule name
     * @return claimable amount
     */
    function getClaimable(address account, string calldata scheduleName) external view returns (uint128) {
        return _getClaimable(schedules[scheduleName], allocations[account][scheduleName]);
    }

    /**
     * @dev Claim claimable tokens, this will mint tokens.
     * @param account the account to claim for
     * @param scheduleName the schedule name
     */
    function _claim(address account, string memory scheduleName) internal {
        Allocation storage allocation = allocations[account][scheduleName];
        require(allocation.amount > 0, "There is nothing to claim");
        require(!allocation.revoked, "Account is revoked");

        Schedule storage schedule = schedules[scheduleName];

        uint128 claimable = _getClaimable(schedule, allocation);
        // Trying to claim more than allocated
        assert(claimable <= allocation.amount);

        uint128 newClaimed = _addUint128(allocation.claimed, claimable);
        // Trying to claim more than allocated
        assert(claimable <= newClaimed);

        uint128 newTotalMinted = _addUint128(totalMinted, claimable);
        // Total amount minted should be less or equal than specified
        // we only check this when a user claims, not when allocations
        // are added
        assert(newTotalMinted <= maxMintAmount);

        totalMinted = newTotalMinted;
        allocation.claimed = newClaimed;
        allocation.lastClaim = _currentBlockTimestamp();

        // mint tokens
        token.mint(account, claimable, "", "");

        emit Claimed(account, claimable, scheduleName);
    }

    /**
     * @dev Calculates claimable tokens.
     * This function expects that the owner has added the schedule
     * periods in ascending order.
     */
    function _getClaimable(
        Schedule storage schedule,
        Allocation storage allocation
    ) internal view returns (uint128) {
        // first unlock hasn't passed yet
        if (_addUint32(startTime, schedule.durations[0]) > _currentBlockTimestamp()) {
            return 0;
        }

        // last unlock has passed
        if (_addUint32(startTime, schedule.durations[schedule.durations.length - 1]) < _currentBlockTimestamp()) {
            // make sure to exclude already claimed amount
            return _subUint128(allocation.amount, allocation.claimed);
        }

        uint128 claimable = 0;

        for (uint256 i = 0; i < schedule.durations.length; i++) {
            uint32 scheduleDeadline = _addUint32(startTime, schedule.durations[i]);

            // schedule deadline not passed, exiting
            if (scheduleDeadline > _currentBlockTimestamp()) break;
            // already claimed during this period, skipping
            if (allocation.lastClaim > scheduleDeadline) continue;

            claimable = _addUint128(claimable, _divUint128(_mulUint128(allocation.amount, schedule.percents[i]), MULTIPLIER));
        }

        return claimable;
    }

    function _currentBlockTimestamp() internal view returns (uint32) {
        // solhint-disable-next-line
        return uint32(block.timestamp % 2 ** 32);
    }

    // SafeMath variations
    function _addUint32(uint32 a, uint32 b) internal pure returns (uint32) {
        uint32 c = a + b;
        require(c >= a, "uint32 addition overflow");

        return c;
    }

    function _addUint128(uint128 a, uint128 b) internal pure returns (uint128) {
        uint128 c = a + b;
        require(c >= a, "uint128 addition overflow");

        return c;
    }

    function _subUint128(uint128 a, uint128 b) internal pure returns (uint128) {
        require(b <= a, "uint128 subtraction overflow");
        uint128 c = a - b;

        return c;
    }

    function _mulUint128(uint128 a, uint128 b) internal pure returns (uint128) {
        if (a == 0) {
            return 0;
        }

        uint128 c = a * b;
        require(c / a == b, "uint128 multiplication overflow");

        return c;
    }

    function _divUint128(uint128 a, uint128 b) internal pure returns (uint128) {
        require(b > 0, "uint128 division by zero");
        uint128 c = a / b;

        return c;
    }

    event ScheduleAdded(uint32[] durations, uint32[] percents, string name);
    event AllocationAdded(address indexed account, uint128 amount, string scheduleName);
    event Claimed(address indexed account, uint128 amount, string scheduleName);
}
