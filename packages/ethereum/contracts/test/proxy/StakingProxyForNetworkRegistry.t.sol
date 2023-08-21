// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { HoprStakingProxyForNetworkRegistry } from "../../src/proxy/StakingProxyForNetworkRegistry.sol";
import "forge-std/Test.sol";

contract HoprStakingProxyForNetworkRegistryTest is Test {
    HoprStakingProxyForNetworkRegistry public hoprStakingProxyForNetworkRegistry;
    address public owner;
    address public stakeContract;

    uint256[] public NFT_TYPE = [1, 2];
    string[] public NFT_RANK = ["123", "456"];
    uint256 public constant INITIAL_MIN_STAKE = 1000;
    uint256 public constant HIGH_STAKE = 2000;
    uint256 public constant LOW_STAKE = 100;
    uint256 public constant SPECIAL_NFT_TYPE_INDEX = 3; // 'Network_registry'
    string public constant SPECIAL_NFT_TYPE_NAME = "Network_registry";
    string public constant SPECIAL_NFT_RANK_TECH = "developer";
    string public constant SPECIAL_NFT_RANK_COM = "community";
    uint256 public constant MAX_REGISTRATION_TECH = type(uint256).max;
    uint256 public constant MAX_REGISTRATION_COM = 1;
    address[] public accounts = new address[](7);

    /**
     * Manually import the errors and events
     */
    event NftTypeAndRankAdded(uint256 indexed nftType, string nftRank); // emit when a new NFT type and rank gets
        // included in the eligibility list
    event NftTypeAndRankRemoved(uint256 indexed nftType, string nftRank); // emit when a NFT type and rank gets removed
        // from the eligibility list
    event SpecialNftTypeAndRankAdded(uint256 indexed nftType, string nftRank, uint256 indexed maxRegistration); // emit
        // when a new special type and rank of NFT gets included in the eligibility list
    event SpecialNftTypeAndRankRemoved(uint256 indexed nftType, string nftRank); // emit when a special type and rank of
        // NFT gets removed from the eligibility list
    event ThresholdUpdated(uint256 indexed threshold); // emit when the staking threshold gets updated.
    event StakeContractUpdated(address indexed stakeContract); // emit when the staking threshold gets updated.

    function setUp() public virtual {
        stakeContract = vm.addr(100); // make vm.addr(100) stakeContract
        owner = vm.addr(101); // make address(101) new owner
        // set _minStake with the production value
        hoprStakingProxyForNetworkRegistry = new HoprStakingProxyForNetworkRegistry(
      stakeContract,
      owner,
      INITIAL_MIN_STAKE
    );
        // assign vm.addr(1) to vm.addr(6) to accounts
        accounts[0] = vm.addr(1);
        accounts[1] = vm.addr(2);
        accounts[2] = vm.addr(3);
        accounts[3] = vm.addr(4);
        accounts[4] = vm.addr(5);
        accounts[5] = vm.addr(6);
        accounts[6] = vm.addr(7);
    }

    /**
     * @dev Check allowance setup is correct
     */
    function test_CheckAllowance() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.prank(owner);
        vm.expectEmit(true, false, false, true, address(hoprStakingProxyForNetworkRegistry));
        emit NftTypeAndRankAdded(NFT_TYPE[0], NFT_RANK[1]);
        hoprStakingProxyForNetworkRegistry.ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1]);

        _helperCheckMaxAllowance([2, 0, 0, 0, 0, 0, 0]);

        vm.clearMockedCalls();
    }

    /**
     * @dev Update threshold:
     * it fails to update with the same threshold
     */
    function testRevert_OwnerRegisterWithWrongArrayLengths() public {
        _helperMockStakeContractReturns();

        vm.prank(owner);
        vm.expectRevert(HoprStakingProxyForNetworkRegistry.SameStakingThreshold.selector);
        hoprStakingProxyForNetworkRegistry.ownerUpdateThreshold(INITIAL_MIN_STAKE);
        vm.clearMockedCalls();
    }

    /**
     * @dev Update threshold:
     * it updates with a different threshold
     */
    function testFuzz_OwnerUpdateRegistry(uint256 newThreshold) public {
        _helperMockStakeContractReturns();
        vm.assume(newThreshold != INITIAL_MIN_STAKE);
        vm.prank(owner);

        vm.expectEmit(true, false, false, false, address(hoprStakingProxyForNetworkRegistry));
        emit ThresholdUpdated(newThreshold);
        hoprStakingProxyForNetworkRegistry.ownerUpdateThreshold(newThreshold);

        vm.clearMockedCalls();
    }

    /**
     * @dev Owner add an existing NFT:
     * it updates with a different threshold
     */
    function testFuzz_OwnerAddExistingNFT() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);
        hoprStakingProxyForNetworkRegistry.ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1]);
        hoprStakingProxyForNetworkRegistry.ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1]);

        _helperCheckMaxAllowance([2, 0, 0, 0, 0, 0, 0]);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Owner batch-add NFTs:
     * it fails to when array length does not match
     */
    function test_OwnerBatchAddNFTs() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);
        // owner has already added one NFT type.
        hoprStakingProxyForNetworkRegistry.ownerAddNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1]);
        uint256[] memory types = new uint256[](3);
        types[0] = NFT_TYPE[0];
        types[1] = NFT_TYPE[1];
        types[2] = NFT_TYPE[1];
        string[] memory ranks = new string[](3);
        ranks[0] = NFT_RANK[1];
        ranks[1] = NFT_RANK[0];
        ranks[2] = NFT_RANK[0];

        hoprStakingProxyForNetworkRegistry.ownerBatchAddNftTypeAndRank(types, ranks);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Update threshold:
     * it fails to when array length does not match
     */
    function testRevert_OwnerBatchAddNFTsWithUnequalLengths() public {
        _helperMockStakeContractReturns();

        uint256[] memory types = new uint256[](3);
        types[0] = NFT_TYPE[0];
        types[1] = NFT_TYPE[1];
        types[2] = NFT_TYPE[1];
        string[] memory ranks = new string[](2);
        ranks[0] = NFT_RANK[1];
        ranks[1] = NFT_RANK[0];
        vm.prank(owner);
        vm.expectRevert(HoprStakingProxyForNetworkRegistry.NftRanksMismatch.selector);
        hoprStakingProxyForNetworkRegistry.ownerBatchAddNftTypeAndRank(types, ranks);
        vm.clearMockedCalls();
    }

    /**
     * @dev Owner remove NFT
     */
    function test_OwnerRemoveNFT() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);
        // update the stake
        hoprStakingProxyForNetworkRegistry.ownerUpdateThreshold(LOW_STAKE);
        // owner has already added a few NFTs.
        uint256[] memory types = new uint256[](2);
        types[0] = NFT_TYPE[0];
        types[1] = NFT_TYPE[1];
        string[] memory ranks = new string[](2);
        ranks[0] = NFT_RANK[1];
        ranks[1] = NFT_RANK[0];
        hoprStakingProxyForNetworkRegistry.ownerBatchAddNftTypeAndRank(types, ranks);

        vm.expectEmit(true, false, false, true, address(hoprStakingProxyForNetworkRegistry));
        emit NftTypeAndRankRemoved(NFT_TYPE[0], NFT_RANK[1]);
        hoprStakingProxyForNetworkRegistry.ownerRemoveNftTypeAndRank(NFT_TYPE[0], NFT_RANK[1]);

        _helperCheckMaxAllowance([0, 20, 0, 1, 20, 0, 0]);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Owner batch-remove NFTs:
     */
    function test_OwnerBatchRemoveNFTs() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);
        // owner has already added one NFT type.
        hoprStakingProxyForNetworkRegistry.ownerAddNftTypeAndRank(NFT_TYPE[1], NFT_RANK[0]);
        uint256[] memory types = new uint256[](2);
        types[0] = NFT_TYPE[0];
        types[1] = NFT_TYPE[1];
        string[] memory ranks = new string[](2);
        ranks[0] = NFT_RANK[1];
        ranks[1] = NFT_RANK[0];

        vm.expectEmit(true, false, false, true, address(hoprStakingProxyForNetworkRegistry));
        emit NftTypeAndRankRemoved(NFT_TYPE[1], NFT_RANK[0]);
        hoprStakingProxyForNetworkRegistry.ownerBatchRemoveNftTypeAndRank(types, ranks);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Owner batch-remove NFTs:
     * it fails to when array length does not match
     */
    function testRevert_OwnerBatchRemoveNFTsWithUnequalLengths() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);
        // owner has already added one NFT type.
        hoprStakingProxyForNetworkRegistry.ownerAddNftTypeAndRank(NFT_TYPE[1], NFT_RANK[0]);
        // update the stake
        hoprStakingProxyForNetworkRegistry.ownerUpdateThreshold(LOW_STAKE);
        uint256[] memory types = new uint256[](2);
        types[0] = NFT_TYPE[0];
        types[1] = NFT_TYPE[1];
        string[] memory ranks = new string[](1);
        ranks[0] = NFT_RANK[0];

        vm.expectRevert(HoprStakingProxyForNetworkRegistry.NftRanksMismatch.selector);
        hoprStakingProxyForNetworkRegistry.ownerBatchRemoveNftTypeAndRank(types, ranks);

        vm.clearMockedCalls();
    }

    /**
     * @dev Fail to add special NFTs due to array length mismatch 1
     */
    function testRevert_OwnerBatchAddSpecialNFTsWrongLength1() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);

        uint256[] memory types = new uint256[](2);
        types[0] = SPECIAL_NFT_TYPE_INDEX;
        types[1] = SPECIAL_NFT_TYPE_INDEX;
        string[] memory ranks = new string[](3);
        ranks[0] = SPECIAL_NFT_RANK_TECH;
        ranks[1] = SPECIAL_NFT_RANK_COM;
        ranks[2] = SPECIAL_NFT_RANK_COM;
        uint256[] memory maxAllownaces = new uint256[](2);
        maxAllownaces[0] = MAX_REGISTRATION_TECH;
        maxAllownaces[1] = MAX_REGISTRATION_COM;

        vm.expectRevert(HoprStakingProxyForNetworkRegistry.NftRanksMismatch.selector);
        hoprStakingProxyForNetworkRegistry.ownerBatchAddSpecialNftTypeAndRank(types, ranks, maxAllownaces);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Fail to add special NFTs due to array length mismatch 2
     */
    function testRevert_OwnerBatchAddSpecialNFTsWrongLength2() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);

        uint256[] memory types = new uint256[](2);
        types[0] = SPECIAL_NFT_TYPE_INDEX;
        types[1] = SPECIAL_NFT_TYPE_INDEX;
        string[] memory ranks = new string[](2);
        ranks[0] = SPECIAL_NFT_RANK_TECH;
        ranks[1] = SPECIAL_NFT_RANK_COM;
        uint256[] memory maxAllownaces = new uint256[](3);
        maxAllownaces[0] = MAX_REGISTRATION_TECH;
        maxAllownaces[1] = MAX_REGISTRATION_COM;
        maxAllownaces[2] = MAX_REGISTRATION_COM;

        vm.expectRevert(HoprStakingProxyForNetworkRegistry.MaxRegistrationsMismatch.selector);
        hoprStakingProxyForNetworkRegistry.ownerBatchAddSpecialNftTypeAndRank(types, ranks, maxAllownaces);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Special NFTs only:
     */
    function test_OwnerBatchAddSpecialNFTs() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);

        uint256[] memory types = new uint256[](2);
        types[0] = SPECIAL_NFT_TYPE_INDEX;
        types[1] = SPECIAL_NFT_TYPE_INDEX;
        string[] memory ranks = new string[](2);
        ranks[0] = SPECIAL_NFT_RANK_TECH;
        ranks[1] = SPECIAL_NFT_RANK_COM;
        uint256[] memory maxAllownaces = new uint256[](2);
        maxAllownaces[0] = MAX_REGISTRATION_TECH;
        maxAllownaces[1] = MAX_REGISTRATION_COM;

        hoprStakingProxyForNetworkRegistry.ownerBatchAddSpecialNftTypeAndRank(types, ranks, maxAllownaces);

        _helperCheckMaxAllowance(
            [uint256(0), uint256(0), MAX_REGISTRATION_TECH, uint256(0), uint256(0), uint256(0), MAX_REGISTRATION_COM]
        );
        // it's possible to overwrite them
        hoprStakingProxyForNetworkRegistry.ownerBatchAddSpecialNftTypeAndRank(types, ranks, maxAllownaces);
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Special NFTs on top of normal nfts:
     */
    function test_NormalNftsAndSpecialNFTs() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);
        // owner has already added one NFT type.
        hoprStakingProxyForNetworkRegistry.ownerAddNftTypeAndRank(NFT_TYPE[1], NFT_RANK[0]);

        uint256[] memory types = new uint256[](2);
        types[0] = SPECIAL_NFT_TYPE_INDEX;
        types[1] = SPECIAL_NFT_TYPE_INDEX;
        string[] memory ranks = new string[](2);
        ranks[0] = SPECIAL_NFT_RANK_TECH;
        ranks[1] = SPECIAL_NFT_RANK_COM;
        uint256[] memory maxAllownaces = new uint256[](2);
        maxAllownaces[0] = MAX_REGISTRATION_TECH;
        maxAllownaces[1] = MAX_REGISTRATION_COM;

        hoprStakingProxyForNetworkRegistry.ownerBatchAddSpecialNftTypeAndRank(types, ranks, maxAllownaces);

        _helperCheckMaxAllowance(
            [uint256(0), uint256(2), MAX_REGISTRATION_TECH, uint256(0), uint256(2), uint256(0), MAX_REGISTRATION_COM]
        );

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Both special NFTs on top of normal nfts:
     */
    function test_NormalNftsAndTwoSpecialNFTs() public {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);
        // owner has already added one NFT type.
        hoprStakingProxyForNetworkRegistry.ownerAddNftTypeAndRank(NFT_TYPE[1], NFT_RANK[0]);
        // account[6] also has TECH NFT
        vm.mockCall(
            stakeContract,
            abi.encodeWithSignature(
                "isNftTypeAndRankRedeemed2(uint256,string,address)",
                SPECIAL_NFT_TYPE_INDEX,
                SPECIAL_NFT_RANK_TECH,
                accounts[6]
            ),
            abi.encode(true)
        );

        uint256[] memory types = new uint256[](2);
        types[0] = SPECIAL_NFT_TYPE_INDEX;
        types[1] = SPECIAL_NFT_TYPE_INDEX;
        string[] memory ranks = new string[](2);
        ranks[0] = SPECIAL_NFT_RANK_TECH;
        ranks[1] = SPECIAL_NFT_RANK_COM;
        uint256[] memory maxAllownaces = new uint256[](2);
        maxAllownaces[0] = MAX_REGISTRATION_TECH;
        maxAllownaces[1] = MAX_REGISTRATION_COM;

        hoprStakingProxyForNetworkRegistry.ownerBatchAddSpecialNftTypeAndRank(types, ranks, maxAllownaces);

        _helperCheckMaxAllowance(
            [uint256(0), uint256(2), MAX_REGISTRATION_TECH, uint256(0), uint256(2), uint256(0), MAX_REGISTRATION_TECH]
        );

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev canOperateFor is always true
     */
    function testFuzz_MaxAllowedRegistrations(address account, address nodeAddress) public {
        assertTrue(hoprStakingProxyForNetworkRegistry.canOperateFor(account, nodeAddress));
        vm.prank(owner);
    }

    /**
     * @dev Owner fail to remove special NFTs in batch due to mismatched length
     */
    function testRevert_OwnerBatchRemoveSpecialNFTs() public {
        _helperRegisterNodes();
        uint256[] memory types = new uint256[](2);
        types[0] = SPECIAL_NFT_TYPE_INDEX;
        types[1] = SPECIAL_NFT_TYPE_INDEX;
        string[] memory ranks = new string[](3);
        ranks[0] = SPECIAL_NFT_RANK_TECH;
        ranks[1] = SPECIAL_NFT_RANK_COM;
        ranks[2] = SPECIAL_NFT_RANK_COM;
        vm.prank(owner);
        vm.expectRevert(HoprStakingProxyForNetworkRegistry.NftRanksMismatch.selector);
        hoprStakingProxyForNetworkRegistry.ownerBatchRemoveSpecialNftTypeAndRank(types, ranks);
        vm.clearMockedCalls();
    }

    /**
     * @dev Owner fail to remove special NFTs in batch due to mismatched length
     */
    function test_OwnerBatchRemoveSpecialNFTs() public {
        _helperRegisterNodes();
        uint256[] memory types = new uint256[](2);
        types[0] = SPECIAL_NFT_TYPE_INDEX;
        types[1] = SPECIAL_NFT_TYPE_INDEX;
        string[] memory ranks = new string[](2);
        ranks[0] = SPECIAL_NFT_RANK_TECH;
        ranks[1] = SPECIAL_NFT_RANK_COM;

        vm.prank(owner);
        vm.expectEmit(true, false, false, true, address(hoprStakingProxyForNetworkRegistry));
        emit SpecialNftTypeAndRankRemoved(types[0], ranks[0]);
        vm.expectEmit(true, false, false, true, address(hoprStakingProxyForNetworkRegistry));
        emit SpecialNftTypeAndRankRemoved(types[1], ranks[1]);
        hoprStakingProxyForNetworkRegistry.ownerBatchRemoveSpecialNftTypeAndRank(types, ranks);
        vm.clearMockedCalls();
    }

    /**
     * @dev Owner can update the staking account
     */
    function test_updateStakeContract(address newStaking) public {
        vm.prank(owner);
        vm.expectEmit(true, false, false, false, address(hoprStakingProxyForNetworkRegistry));
        emit StakeContractUpdated(newStaking);
        hoprStakingProxyForNetworkRegistry.updateStakeContract(newStaking);
    }

    /**
     * @dev Helper function to mock the value of stake
     * for accounts from vm.addr(1) to vm.addr(7)
     * Allocation of NFTs and staks
     * | NFT Type   | 0 | 0 | 1 | 1 | Network_registry | Network_registry | Stake |
     * |------------|---|---|---|---|------------------|------------------|-------|
     * | NFT Rank   | 0 | 1 | 0 | 1 | developer        | community        | --    |
     * |------------|---|---|---|---|------------------|------------------|-------|
     * | vm.addr(1) |   | x |   |   |                  |                  | 2000  |
     * | vm.addr(2) |   |   | x |   |                  |                  | 2000  |
     * | vm.addr(3) |   | x |   |   | x                |                  | 100   |
     * | vm.addr(4) |   |   | x |   |                  |                  | 100   |
     * | vm.addr(5) |   |   | x |   |                  |                  | 2000  |
     * | vm.addr(6) |   | x |   |   |                  |                  | 100   |
     * | vm.addr(7) |   |   | x |   |                  | x                | 0     |
     * |------------|---|---|---|---|------------------|------------------|-------|
     */
    function _helperMockStakeContractReturns() internal {
        // allocate NFTs and stakes for 7 accounts;
        _helperAllocateNftAndStake(accounts[0], false, true, false, false, HIGH_STAKE);
        _helperAllocateNftAndStake(accounts[1], false, false, true, false, HIGH_STAKE);
        _helperAllocateNftAndStake(accounts[2], false, true, false, false, LOW_STAKE);
        _helperAllocateNftAndStake(accounts[3], false, false, true, false, LOW_STAKE);
        _helperAllocateNftAndStake(accounts[4], false, false, true, false, HIGH_STAKE);
        _helperAllocateNftAndStake(accounts[5], false, true, false, false, LOW_STAKE);
        _helperAllocateNftAndStake(accounts[6], false, false, true, false, 0);

        vm.mockCall(
            stakeContract,
            abi.encodeWithSignature("isNftTypeAndRankRedeemed2(uint256,string,address)"),
            abi.encode(false)
        );
        for (uint256 index = 0; index < accounts.length; index++) {
            // most accounts don't have special NFTs redeemed (TECH), except for account[2]
            vm.mockCall(
                stakeContract,
                abi.encodeWithSignature(
                    "isNftTypeAndRankRedeemed2(uint256,string,address)",
                    SPECIAL_NFT_TYPE_INDEX,
                    SPECIAL_NFT_RANK_TECH,
                    accounts[index]
                ),
                abi.encode(index == 2 ? true : false)
            );
            // most accounts don't have special NFTs redeemed (COM), except for account[6]
            vm.mockCall(
                stakeContract,
                abi.encodeWithSignature(
                    "isNftTypeAndRankRedeemed2(uint256,string,address)",
                    SPECIAL_NFT_TYPE_INDEX,
                    SPECIAL_NFT_RANK_COM,
                    accounts[index]
                ),
                abi.encode(index == 6 ? true : false)
            );
        }
    }

    function _helperAllocateNftAndStake(
        address account,
        bool hasType0Rank0,
        bool hasType0Rank1,
        bool hasType1Rank0,
        bool hasType1Rank1,
        uint256 stake
    )
        internal
    {
        // return if an account has certain NFTs
        vm.mockCall(
            stakeContract,
            abi.encodeWithSignature(
                "isNftTypeAndRankRedeemed2(uint256,string,address)", NFT_TYPE[0], NFT_RANK[0], account
            ),
            abi.encode(hasType0Rank0)
        );
        vm.mockCall(
            stakeContract,
            abi.encodeWithSignature(
                "isNftTypeAndRankRedeemed2(uint256,string,address)", NFT_TYPE[0], NFT_RANK[1], account
            ),
            abi.encode(hasType0Rank1)
        );
        vm.mockCall(
            stakeContract,
            abi.encodeWithSignature(
                "isNftTypeAndRankRedeemed2(uint256,string,address)", NFT_TYPE[1], NFT_RANK[0], account
            ),
            abi.encode(hasType1Rank0)
        );
        vm.mockCall(
            stakeContract,
            abi.encodeWithSignature(
                "isNftTypeAndRankRedeemed2(uint256,string,address)", NFT_TYPE[1], NFT_RANK[1], account
            ),
            abi.encode(hasType1Rank1)
        );
        // return stake value
        vm.mockCall(stakeContract, abi.encodeWithSignature("stakedHoprTokens(address)", account), abi.encode(stake));
    }

    function _helperCheckMaxAllowance(uint8[7] memory allowances) internal {
        for (uint256 i = 0; i < accounts.length; i++) {
            assertEq(hoprStakingProxyForNetworkRegistry.maxAllowedRegistrations(accounts[i]), allowances[i]);
        }
    }

    function _helperCheckMaxAllowance(uint256[7] memory allowances) internal {
        for (uint256 i = 0; i < accounts.length; i++) {
            assertEq(hoprStakingProxyForNetworkRegistry.maxAllowedRegistrations(accounts[i]), allowances[i]);
        }
    }

    function _helperRegisterNodes() private {
        _helperMockStakeContractReturns();
        // owner add nft type and rank
        vm.startPrank(owner);

        uint256[] memory types = new uint256[](2);
        types[0] = SPECIAL_NFT_TYPE_INDEX;
        types[1] = SPECIAL_NFT_TYPE_INDEX;
        string[] memory ranks = new string[](2);
        ranks[0] = SPECIAL_NFT_RANK_TECH;
        ranks[1] = SPECIAL_NFT_RANK_COM;
        uint256[] memory maxAllownaces = new uint256[](2);
        maxAllownaces[0] = MAX_REGISTRATION_TECH;
        maxAllownaces[1] = MAX_REGISTRATION_COM;

        hoprStakingProxyForNetworkRegistry.ownerBatchAddSpecialNftTypeAndRank(types, ranks, maxAllownaces);

        _helperCheckMaxAllowance(
            [uint256(0), uint256(0), MAX_REGISTRATION_TECH, uint256(0), uint256(0), uint256(0), MAX_REGISTRATION_COM]
        );
        vm.stopPrank();
    }
}
