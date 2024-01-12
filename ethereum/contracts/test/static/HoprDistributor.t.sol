// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import "../../src/static/HoprDistributor.sol";
import "../../src/static/HoprToken.sol";
import "../utils/ERC1820Registry.sol";
import "forge-std/Test.sol";

contract HoprDistributorTest is Test, ERC1820RegistryFixtureTest {
    HoprDistributor public hoprDistributor;

    string public SCHEDULE_UNSET = "SCHEDULE_UNSET";
    string public SCHEDULE_1_MIN_ALL = "SCHEDULE_1_MIN_ALL";
    string public SCHEDULE_TEAM = "SCHEDULE_TEAM";
    uint128 public DEFAULT_MAX_MINT = 500;
    address public newOwner;
    address public tokenAddress;
    uint128 public startTime;

    /**
     * Manually import the errors and events
     */
    event ScheduleAdded(uint128[] durations, uint128[] percents, string name);
    event AllocationAdded(address indexed account, uint128 amount, string scheduleName);
    event Claimed(address indexed account, uint128 amount, string scheduleName);

    function setUp() public virtual override {
        super.setUp();
        startTime = uint128(block.timestamp + 5 * 60); // 5 minutes after the contract deployment;
        tokenAddress = vm.addr(1); // make vm.addr(1) HoprToken contract
        newOwner = vm.addr(100); // make address(100) owner
        vm.prank(newOwner);
        hoprDistributor = new HoprDistributor(HoprToken(tokenAddress), startTime, DEFAULT_MAX_MINT);
    }

    /**
     * @dev it should update start time
     */
    function test_StartTime(uint128 newStartTime) public {
        vm.assume(block.timestamp < startTime);
        assertEq(hoprDistributor.startTime(), startTime);
        // allow update start time
        vm.prank(newOwner);
        hoprDistributor.updateStartTime(newStartTime);
        assertEq(hoprDistributor.startTime(), newStartTime);
    }

    /**
     * @dev it should fail to update start time
     */
    function testRevert_WhenDistributorHasStarted(uint128 newStartTime) public {
        skip(6 * 60); // skip 6 minutes
        vm.prank(newOwner);
        vm.expectRevert("Previous start time must not be reached");
        hoprDistributor.updateStartTime(newStartTime);
    }

    /**
     * @dev it should add schedule
     */
    function test_Schedule() public {
        uint128[] memory durations = new uint128[](1);
        durations[0] = 60;
        uint128[] memory percents = new uint128[](1);
        percents[0] = hoprDistributor.MULTIPLIER();
        vm.prank(newOwner);
        vm.expectEmit(false, false, false, true, address(hoprDistributor));
        emit ScheduleAdded(durations, percents, SCHEDULE_1_MIN_ALL);
        hoprDistributor.addSchedule(durations, percents, SCHEDULE_1_MIN_ALL);

        uint256 claimable = hoprDistributor.getClaimable(newOwner, SCHEDULE_1_MIN_ALL);
        assertEq(claimable, 0);

        (uint128[] memory dur, uint128[] memory per) = hoprDistributor.getSchedule(SCHEDULE_1_MIN_ALL);
        assertEq(dur.length, 1);
        assertEq(per.length, 1);
        assertEq(dur[0], durations[0]);
        assertEq(per[0], per[0]);
    }

    /**
     * @dev it should fail to add schedule again
     */
    function testRevert_WhenAddScheduleAgain() public {
        _helperAddBasicSchedule();
        uint128[] memory durations;
        uint128[] memory percents;
        vm.prank(newOwner);
        vm.expectRevert("Schedule must not exist");
        hoprDistributor.addSchedule(durations, percents, SCHEDULE_1_MIN_ALL);
    }

    /**
     * @dev it should fail to add schedule with mismatching inputs
     */
    function testRevert_WhenMismatchInputsForSchedule() public {
        _helperAddBasicSchedule();
        uint128[] memory durations = new uint128[](1);
        durations[0] = 1;
        uint128[] memory percents;
        vm.prank(newOwner);
        vm.expectRevert("Durations and percents must have equal length");
        hoprDistributor.addSchedule(durations, percents, SCHEDULE_UNSET);
    }

    /**
     * @dev it should fail to add schedule when durations are not in ascending order
     */
    function testRevert_WhenNotAscendingOrder() public {
        _helperAddBasicSchedule();
        uint128[] memory durations = new uint128[](2);
        durations[0] = 5;
        durations[1] = 1;
        uint128[] memory percents = new uint128[](2);
        percents[0] = 50;
        percents[1] = 50;
        vm.prank(newOwner);
        vm.expectRevert("Durations must be added in ascending order");
        hoprDistributor.addSchedule(durations, percents, SCHEDULE_UNSET);
    }

    /**
     * @dev it should fail to add schedule when percent is higher than multiplier
     */
    function testRevert_WhenHigherThanMultiplier() public {
        _helperAddBasicSchedule();
        uint128[] memory durations = new uint128[](2);
        durations[0] = 1;
        durations[1] = 2;
        uint128[] memory percents = new uint128[](2);
        percents[0] = 50;
        percents[1] = hoprDistributor.MULTIPLIER() + 1;
        vm.prank(newOwner);
        vm.expectRevert("Percent provided must be smaller or equal to MULTIPLIER");
        hoprDistributor.addSchedule(durations, percents, SCHEDULE_UNSET);
    }

    /**
     * @dev it should fail to add schedule when percents do not sum to 100%
     */
    function testRevert_WhenSumNotTo100(uint128 val) public {
        val = uint128(bound(val, 0, hoprDistributor.MULTIPLIER() - 1));
        _helperAddBasicSchedule();
        uint128[] memory durations = new uint128[](1);
        durations[0] = 60;
        uint128[] memory percents = new uint128[](1);
        percents[0] = val;
        vm.prank(newOwner);
        vm.expectRevert("Percents must sum to MULTIPLIER amount");
        hoprDistributor.addSchedule(durations, percents, SCHEDULE_UNSET);
    }

    /**
     * @dev it should fail to add allocation when schedule does not exist
     */
    function testRevert_WhenScheduleNotExistCannotAddAllocations() public {
        address[] memory accounts;
        uint128[] memory amounts;
        vm.prank(newOwner);
        vm.expectRevert("Schedule must exist");
        hoprDistributor.addAllocations(accounts, amounts, SCHEDULE_1_MIN_ALL);
    }

    /**
     * @dev it should add allocation
     */
    function test_AddAllocations() public {
        _helperAddBasicSchedule();
        address[] memory accounts = new address[](1);
        accounts[0] = newOwner;
        uint128[] memory amounts = new uint128[](1);
        amounts[0] = 100;
        vm.prank(newOwner);
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit AllocationAdded(accounts[0], amounts[0], SCHEDULE_1_MIN_ALL);
        hoprDistributor.addAllocations(accounts, amounts, SCHEDULE_1_MIN_ALL);

        assertEq(hoprDistributor.totalToBeMinted(), amounts[0]);
        assertEq(hoprDistributor.getClaimable(accounts[0], SCHEDULE_1_MIN_ALL), 0);
    }

    /**
     * @dev It should fail to add allocation with mismatching inputs
     */
    function testRevert_WhenMismatchInputsForAllocation() public {
        _helperAddBasicSchedule();
        address[] memory accounts = new address[](1);
        accounts[0] = newOwner;
        uint128[] memory amounts;
        vm.prank(newOwner);
        vm.expectRevert("Accounts and amounts must have equal length");
        hoprDistributor.addAllocations(accounts, amounts, SCHEDULE_1_MIN_ALL);
    }

    /**
     * @dev It should fail to add allocation again
     */
    function testRevert_WhenAddAllocationsAgain() public {
        _helperAddBasicSchedule();
        _helperAddAllocation(SCHEDULE_1_MIN_ALL);
        address[] memory accounts = new address[](1);
        accounts[0] = newOwner;
        uint128[] memory amounts = new uint128[](1);
        amounts[0] = 100;
        vm.prank(newOwner);
        vm.expectRevert("Allocation must not exist");
        hoprDistributor.addAllocations(accounts, amounts, SCHEDULE_1_MIN_ALL);
    }

    function test_AddSecondAllocation() public {
        _helperAddBasicSchedule();
        _helperAddAllocation(SCHEDULE_1_MIN_ALL);

        uint128[] memory durations = new uint128[](1);
        durations[0] = 60;
        uint128[] memory percents = new uint128[](1);
        percents[0] = hoprDistributor.MULTIPLIER();
        address[] memory accounts = new address[](1);
        accounts[0] = newOwner;
        uint128[] memory amounts = new uint128[](1);
        amounts[0] = 200;

        vm.startPrank(newOwner);
        hoprDistributor.addSchedule(durations, percents, SCHEDULE_TEAM);
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit AllocationAdded(accounts[0], amounts[0], SCHEDULE_TEAM);
        hoprDistributor.addAllocations(accounts, amounts, SCHEDULE_TEAM);
        vm.stopPrank();

        assertEq(hoprDistributor.totalToBeMinted(), 300);
        assertEq(hoprDistributor.getClaimable(accounts[0], SCHEDULE_TEAM), 0);
    }

    /**
     * @dev Claimable
     * it should be able to claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL
     * it should be able to claim 0 after 2 minutes using SCHEDULE_TEAM
     * it should be able to claim 12 after 5 minutes using SCHEDULE_TEAM
     * it should be able to claim 24 after 8 minutes using SCHEDULE_TEAM
     * it should be able to claim 100 after 19 minutes using SCHEDULE_TEAM
     */
    function test_Claimable() public {
        _helperClaimableSchedulesAndAllocations();
        // skip to the start of the distributor
        vm.warp(hoprDistributor.startTime());
        skip(2 * 60); // skip 2 minutes
        // it should be able to claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_1_MIN_ALL), 100);
        // it should be able to claim 0 after 2 minutes using SCHEDULE_TEAM
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 0);
        // it should be able to claim 12 after 5 minutes using SCHEDULE_TEAM
        skip(2 * 60); // skip 2 minutes
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 12);
        // it should be able to claim 24 after 8 minutes using SCHEDULE_TEAM
        skip(2 * 60); // skip 2 minutes
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 24);
        // it should be able to claim 100 after 19 minutes using SCHEDULE_TEAM
        skip(12 * 60 + 1); // skip 12 minutes
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 100);
    }

    /**
     * @dev Claim
     * it should claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL
     * it should claim 0 after 2 minutes using SCHEDULE_TEAM
     * it should claim 12 after 5 minutes using SCHEDULE_TEAM
     * it should claim 24 after 8 minutes using SCHEDULE_TEAM
     * it should claim 100 after 19 minutes using SCHEDULE_TEAM
     */
    function test_Claim() public {
        _helperClaimableSchedulesAndAllocations();
        // skip to the start of the distributor
        vm.warp(hoprDistributor.startTime());
        vm.startPrank(newOwner);

        // it should claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL
        skip(2 * 60); // skip 2 minutes
        vm.mockCall(
            tokenAddress,
            abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", address(newOwner), 100, hex"00", hex"00"),
            abi.encode(true)
        );
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit Claimed(newOwner, 100, SCHEDULE_1_MIN_ALL);
        hoprDistributor.claim(SCHEDULE_1_MIN_ALL);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_1_MIN_ALL), 0);

        // it should claim 0 after 2 minutes using SCHEDULE_TEAM
        hoprDistributor.claim(SCHEDULE_TEAM);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 0);

        // it should claim 12 after 5 minutes using SCHEDULE_TEAM
        skip(2 * 60); // skip 2 minutes
        vm.mockCall(
            tokenAddress,
            abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", address(newOwner), 12, hex"00", hex"00"),
            abi.encode(true)
        );
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit Claimed(newOwner, 12, SCHEDULE_TEAM);
        hoprDistributor.claim(SCHEDULE_TEAM);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 0);

        // it should claim 24 after 8 minutes using SCHEDULE_TEAM
        skip(2 * 60); // skip 2 minutes
        vm.mockCall(
            tokenAddress,
            abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", address(newOwner), 12, hex"00", hex"00"),
            abi.encode(true)
        );
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit Claimed(newOwner, 12, SCHEDULE_TEAM);
        hoprDistributor.claim(SCHEDULE_TEAM);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 0);

        skip(12 * 60 + 1); // skip 12 minutes
        // it should claim 100 after 19 minutes using SCHEDULE_TEAM
        vm.mockCall(
            tokenAddress,
            abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", address(newOwner), 76, hex"00", hex"00"),
            abi.encode(true)
        );
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit Claimed(newOwner, 76, SCHEDULE_TEAM);
        hoprDistributor.claim(SCHEDULE_TEAM);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 0);
    }

    /**
     * @dev it should fail when there's nothing to claim
     */
    function testRevert_ClaimWhenNothingToClaim() public {
        _helperClaimableSchedulesAndAllocations();
        // skip to the start of the distributor
        vm.warp(hoprDistributor.startTime());
        vm.prank(newOwner);
        vm.expectRevert("There is nothing to claim");
        hoprDistributor.claim(SCHEDULE_UNSET);
    }

    /**
     * @dev ClaimFor
     * it should claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL
     * it should claim 0 after 2 minutes using SCHEDULE_TEAM
     * it should claim 12 after 5 minutes using SCHEDULE_TEAM
     * it should claim 24 after 8 minutes using SCHEDULE_TEAM
     * it should claim 100 after 19 minutes using SCHEDULE_TEAM
     */
    function test_ClaimFor(address caller) public {
        vm.assume(caller != newOwner);
        _helperClaimableSchedulesAndAllocations();
        // skip to the start of the distributor
        vm.warp(hoprDistributor.startTime());
        vm.startPrank(caller);

        // it should claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL
        skip(2 * 60); // skip 2 minutes
        vm.mockCall(
            tokenAddress,
            abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", address(newOwner), 100, hex"00", hex"00"),
            abi.encode(true)
        );
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit Claimed(newOwner, 100, SCHEDULE_1_MIN_ALL);
        hoprDistributor.claimFor(newOwner, SCHEDULE_1_MIN_ALL);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_1_MIN_ALL), 0);

        // it should claim 0 after 2 minutes using SCHEDULE_TEAM
        hoprDistributor.claimFor(newOwner, SCHEDULE_TEAM);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 0);

        // it should claim 12 after 5 minutes using SCHEDULE_TEAM
        skip(2 * 60); // skip 2 minutes
        vm.mockCall(
            tokenAddress,
            abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", address(newOwner), 12, hex"00", hex"00"),
            abi.encode(true)
        );
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit Claimed(newOwner, 12, SCHEDULE_TEAM);
        hoprDistributor.claimFor(newOwner, SCHEDULE_TEAM);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 0);

        // it should claim 24 after 8 minutes using SCHEDULE_TEAM
        skip(2 * 60); // skip 2 minutes
        vm.mockCall(
            tokenAddress,
            abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", address(newOwner), 12, hex"00", hex"00"),
            abi.encode(true)
        );
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit Claimed(newOwner, 12, SCHEDULE_TEAM);
        hoprDistributor.claimFor(newOwner, SCHEDULE_TEAM);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 0);

        skip(12 * 60 + 1); // skip 12 minutes
        // it should claim 100 after 19 minutes using SCHEDULE_TEAM
        vm.mockCall(
            tokenAddress,
            abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", address(newOwner), 76, hex"00", hex"00"),
            abi.encode(true)
        );
        vm.expectEmit(true, false, false, true, address(hoprDistributor));
        emit Claimed(newOwner, 76, SCHEDULE_TEAM);
        hoprDistributor.claimFor(newOwner, SCHEDULE_TEAM);
        assertEq(hoprDistributor.getClaimable(newOwner, SCHEDULE_TEAM), 0);
    }

    /**
     * @dev it should fail when there's nothing to claim for
     */
    function testRevert_ClaimWhenNothingToClaim(address caller) public {
        vm.assume(caller != newOwner);
        _helperClaimableSchedulesAndAllocations();
        // skip to the start of the distributor
        vm.warp(hoprDistributor.startTime());
        vm.prank(caller);

        vm.expectRevert("There is nothing to claim");
        hoprDistributor.claimFor(newOwner, SCHEDULE_UNSET);
    }

    /**
     * @dev Revoke
     * it should fail to claim SCHEDULE_1_MIN_ALL after revoked
     */
    function testRevert_Revoke() public {
        _helperClaimableSchedulesAndAllocations();
        // skip to the start of the distributor
        vm.warp(hoprDistributor.startTime());
        vm.startPrank(newOwner);
        hoprDistributor.revokeAccount(newOwner, SCHEDULE_1_MIN_ALL);
        assertEq(hoprDistributor.totalToBeMinted(), 100);

        vm.expectRevert("Account is revoked");
        hoprDistributor.claim(SCHEDULE_1_MIN_ALL);
    }

    /**
     * @dev Revoke
     * it should fail to claim SCHEDULE_TEAM after revoked
     */
    function testRevert_ClaimWhenScheduleIsRevoked() public {
        _helperClaimableSchedulesAndAllocations();
        // skip to the start of the distributor
        vm.warp(hoprDistributor.startTime());
        skip(2 * 60); // skip 2 minutes
        vm.startPrank(newOwner);
        // claim SCHEDULE_TEAM
        vm.mockCall(
            tokenAddress,
            abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", address(newOwner), 12, hex"00", hex"00"),
            abi.encode(true)
        );
        hoprDistributor.claim(SCHEDULE_TEAM);
        // revoke account
        hoprDistributor.revokeAccount(newOwner, SCHEDULE_TEAM);
        assertEq(hoprDistributor.totalToBeMinted(), 100);
        // fail to claim SCHEDULE_TEAM again
        vm.expectRevert("Account is revoked");
        hoprDistributor.claim(SCHEDULE_TEAM);
    }

    /**
     * @dev Revoke
     * it should fail to revoke twice
     * it should fail to revoke if allocation does not exist
     */
    function testRevert_RevokeTwice() public {
        _helperClaimableSchedulesAndAllocations();
        // skip to the start of the distributor
        vm.warp(hoprDistributor.startTime());
        skip(2 * 60); // skip 2 minutes
        vm.startPrank(newOwner);
        // revoke account
        hoprDistributor.revokeAccount(newOwner, SCHEDULE_TEAM);
        // should fail to revoke twice
        vm.expectRevert("Allocation must not be already revoked");
        hoprDistributor.revokeAccount(newOwner, SCHEDULE_TEAM);
        // should fail to revoke if allocation does not exist
        vm.expectRevert("Allocation must exist");
        hoprDistributor.revokeAccount(newOwner, SCHEDULE_UNSET);
    }

    /**
     * @dev it should fail to allocate if totalToBeMinted is higher than max mint
     * FIXME: Cannot catch uint128 arithmetic overflow. Encountered error
     * [FAIL. Reason: Error != expected error: NH{q != NH{q Counterexample:
     * calldata=0x224dafb90000000000000000000000000000000000000000000000000000000000000000, args=[0]]
     */
    function testFail_ExceedMaxMint(uint128 amount) public {
        _helperAddBasicSchedule();
        amount = uint128(bound(amount, DEFAULT_MAX_MINT + 1, 1e20));

        address[] memory accounts = new address[](1);
        accounts[0] = newOwner;
        uint128[] memory amounts = new uint128[](1);
        amounts[0] = amount;

        vm.prank(newOwner);
        // vm.expectRevert(stdError.arithmeticError);
        hoprDistributor.addAllocations(accounts, amounts, SCHEDULE_1_MIN_ALL);
    }

    /**
     * @dev helper function to add SCHEDULE_1_MIN_ALL schedule
     */
    function _helperAddBasicSchedule() internal {
        uint128[] memory durations = new uint128[](1);
        durations[0] = 60;
        uint128[] memory percents = new uint128[](1);
        percents[0] = hoprDistributor.MULTIPLIER();

        vm.prank(newOwner);
        hoprDistributor.addSchedule(durations, percents, SCHEDULE_1_MIN_ALL);
    }

    /**
     * @dev helper function to add allocations for a schedule
     */
    function _helperAddAllocation(string memory scheduleName) internal {
        address[] memory accounts = new address[](1);
        accounts[0] = newOwner;
        uint128[] memory amounts = new uint128[](1);
        amounts[0] = 100;

        vm.prank(newOwner);
        hoprDistributor.addAllocations(accounts, amounts, scheduleName);
    }

    /**
     * @dev helper function to add SCHEDULE_1_MIN_ALL and SCHEDULE_TEAM schedules
     */
    function _helperClaimableSchedulesAndAllocations() internal {
        _helperAddBasicSchedule();
        uint128[] memory durations = new uint128[](8);
        uint128[] memory percents = new uint128[](8);
        uint128 percentage = hoprDistributor.MULTIPLIER() / 8;
        for (uint256 i = 0; i < durations.length; i++) {
            durations[i] = uint128(60 * (2 * i + 4));
            percents[i] = percentage;
        }
        vm.prank(newOwner);
        hoprDistributor.addSchedule(durations, percents, SCHEDULE_TEAM);

        _helperAddAllocation(SCHEDULE_1_MIN_ALL);
        _helperAddAllocation(SCHEDULE_TEAM);
    }
}
