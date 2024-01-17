// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.19;

import { Ownable } from "openzeppelin-contracts/access/Ownable.sol";
import { Math } from "openzeppelin-contracts/utils/math/Math.sol";
import { IHoprNetworkRegistryRequirement } from "../interfaces/INetworkRegistryRequirement.sol";

/**
 * @dev Interface for staking contract
 * source code at https://github.com/hoprnet/hopr-stake/tree/main/contracts
 * staking v2 is deployed at https://blockscout.com/xdai/mainnet/address/0x2cDD13ddB0346E0F620C8E5826Da5d7230341c6E
 * staking v3 is deployed at https://blockscout.com/xdai/mainnet/address/0xae933331ef0bE122f9499512d3ed4Fa3896DCf20
 */
interface IHoprStake {
    function stakedHoprTokens(address _account) external view returns (uint256);

    function isNftTypeAndRankRedeemed2(
        uint256 nftTypeIndex,
        string memory nftRank,
        address hodler
    )
        external
        view
        returns (bool);
}

/**
 * @dev Proxy for staking (v2/v3/v4) contract, which an "HoprNetworkRegistry requirement" is implemented
 * Two types of accounts are considered eligible:
 * 1) Accounts with HoprBoost NFTs that are of the type and rank in the `eligibleNftTypeAndRank` array
 * are considered as eligible, when their stake is also above the `stakeThreshold`. The maximum allowed
 * registration of these accounts are defined by their stake.
 * 2) Acounts with HoprBoost NFTs of `specialNftTypeAndRank`. Its maximum allowed registration is set
 * by the owner.
 */
contract HoprStakingProxyForNetworkRegistry is IHoprNetworkRegistryRequirement, Ownable {
    using Math for uint256;

    struct NftTypeAndRank {
        uint256 nftType;
        string nftRank;
    }

    IHoprStake public stakeContract; // contract of HoprStake contract
    // minimum amount HOPR tokens being staked in the staking contract to be considered eligible
    // for every stakeThreshold, one peer id can be registered.
    uint256 public stakeThreshold;
    // list of NFTs whose owner are considered as eligible to the network if the `stakeThreshold` is also met
    NftTypeAndRank[] public eligibleNftTypeAndRank;
    // for holders of special NFT, it's the cap of peer ids one address can register.
    uint256[] public maxRegistrationsPerSpecialNft;
    // list of NFTs whose owner are considered as eligible to the network
    // without meeting the `stakeThreshold`, e.g. "Network_registry NFT"
    NftTypeAndRank[] public specialNftTypeAndRank;

    // emit when a new NFT type and rank gets included in the eligibility list
    event NftTypeAndRankAdded(uint256 indexed nftType, string nftRank);
    // emit when a NFT type and rank gets removed from the eligibility list
    event NftTypeAndRankRemoved(uint256 indexed nftType, string nftRank);
    // emit when a new special type and rank of NFT gets included in the eligibility list
    event SpecialNftTypeAndRankAdded(uint256 indexed nftType, string nftRank, uint256 indexed maxRegistration);
    // emit when a special type and rank of NFT gets removed from the eligibility list
    event SpecialNftTypeAndRankRemoved(uint256 indexed nftType, string nftRank);
    // emit when the staking threshold gets updated.
    event ThresholdUpdated(uint256 indexed threshold);
    // emit when the staking threshold gets updated.
    event StakeContractUpdated(address indexed stakeContract);

    error SameStakingThreshold();
    error NftRanksMismatch();
    error MaxRegistrationsMismatch();

    /**
     * @dev Set stake contract address, transfer ownership, and set the maximum registrations per
     * special NFT to the default value: upperbound of of uint256.
     */
    constructor(address _stakeContract, address _newOwner, uint256 _minStake) {
        _updateStakeContract(_stakeContract);
        _transferOwnership(_newOwner);
        stakeThreshold = _minStake;
        emit ThresholdUpdated(stakeThreshold);
    }

    /**
     * @dev Returns the maximum allowed registration
     * a) for each special NFTs staked, consider their `maxRegistrationsPerSpecialNft`
     * b) if NFT of eligibleNftTypeAndRank are redeemed, consider floor(`stake`/`threshold`)
     * returns the maximum of the above mentioned categories
     * @param account staker address that has a hopr nodes running
     */
    function maxAllowedRegistrations(address account) external view returns (uint256) {
        uint256 allowedRegistration;
        // if the account owns a special NFT, requirement is fulfilled
        for (uint256 i = 0; i < specialNftTypeAndRank.length; i++) {
            NftTypeAndRank memory eligible = specialNftTypeAndRank[i];
            if (stakeContract.isNftTypeAndRankRedeemed2(eligible.nftType, eligible.nftRank, account)) {
                allowedRegistration = allowedRegistration.max(maxRegistrationsPerSpecialNft[i]);
            }
        }

        // when no special NFT is present, the account needs to 1) reach the minimum stake, 2) own an eligible NFT
        // for self-claiming accounts, check against the current criteria
        uint256 amount = stakeContract.stakedHoprTokens(account);
        if (amount < stakeThreshold) {
            // threshold does not meet
            return allowedRegistration;
        }
        // check on regular eligible NFTs.
        for (uint256 i = 0; i < eligibleNftTypeAndRank.length; i++) {
            NftTypeAndRank memory eligible = eligibleNftTypeAndRank[i];
            if (stakeContract.isNftTypeAndRankRedeemed2(eligible.nftType, eligible.nftRank, account)) {
                allowedRegistration = allowedRegistration.max(amount / stakeThreshold);
            }
        }

        return allowedRegistration;
    }

    /**
     * @dev Get if the staking account is eligible to act on node address
     */
    function canOperateFor(address, address) external pure returns (bool eligiblity) {
        return true;
    }

    /**
     * @dev Owner adds/updates NFT type and rank to the list of special NFTs in batch.
     * @param nftTypes Array of type indexes of the special HoprBoost NFT
     * @param nftRanks Array of HOPR boost rank, which is associated to the special NFT, in string[]
     * @param maxRegistrations Array of maximum registration per special NFT type
     */
    function ownerBatchAddSpecialNftTypeAndRank(
        uint256[] calldata nftTypes,
        string[] calldata nftRanks,
        uint256[] calldata maxRegistrations
    )
        external
        onlyOwner
    {
        if (nftTypes.length != nftRanks.length) revert NftRanksMismatch();
        if (nftTypes.length != maxRegistrations.length) revert MaxRegistrationsMismatch();
        for (uint256 index = 0; index < nftTypes.length; index++) {
            _addSpecialNftTypeAndRank(nftTypes[index], nftRanks[index], maxRegistrations[index]);
        }
    }

    /**
     * @dev Owner removes from list of special NFTs in batch.
     * @param nftTypes Array of type index of the special HoprBoost NFT
     * @param nftRanks Array of HOPR boost rank, which is associated to the special NFT, in string[]
     */
    function ownerBatchRemoveSpecialNftTypeAndRank(
        uint256[] calldata nftTypes,
        string[] calldata nftRanks
    )
        external
        onlyOwner
    {
        if (nftTypes.length != nftRanks.length) revert NftRanksMismatch();
        for (uint256 index = 0; index < nftTypes.length; index++) {
            _removeSpecialNftTypeAndRank(nftTypes[index], nftRanks[index]);
        }
    }

    /**
     * @dev Owner adds/updates NFT type and rank to the list of eligibles NFTs in batch.
     * @param nftTypes Array of type indexes of the eligible HoprBoost NFT
     * @param nftRanks Array of HOPR boost rank, which is associated to the eligible NFT, in string[]
     */
    function ownerBatchAddNftTypeAndRank(uint256[] calldata nftTypes, string[] calldata nftRanks) external onlyOwner {
        if (nftTypes.length != nftRanks.length) revert NftRanksMismatch();
        for (uint256 index = 0; index < nftTypes.length; index++) {
            _addNftTypeAndRank(nftTypes[index], nftRanks[index]);
        }
    }

    /**
     * @dev Owner removes from list of eligible NFTs in batch.
     * @param nftTypes Array of type index of the eligible HoprBoost NFT
     * @param nftRanks Array of HOPR boost rank, which is associated to the eligible NFT, in string[]
     */
    function ownerBatchRemoveNftTypeAndRank(
        uint256[] calldata nftTypes,
        string[] calldata nftRanks
    )
        external
        onlyOwner
    {
        if (nftTypes.length != nftRanks.length) revert NftRanksMismatch();
        for (uint256 index = 0; index < nftTypes.length; index++) {
            _removeNftTypeAndRank(nftTypes[index], nftRanks[index]);
        }
    }

    /**
     * @dev Owner adds/updates NFT type and rank to the list of eligibles NFTs.
     * @param nftType Type index of the eligible HoprBoost NFT
     * @param nftRank HoprBoost rank which is associated to the eligible NFT, in string
     */
    function ownerAddNftTypeAndRank(uint256 nftType, string memory nftRank) external onlyOwner {
        _addNftTypeAndRank(nftType, nftRank);
    }

    /**
     * @dev Owner removes from list of eligible NFTs
     * @param nftType Type index of the eligible HoprBoost NFT
     * @param nftRank HoprBoost rank which is associated to the eligible NFT, in string
     */
    function ownerRemoveNftTypeAndRank(uint256 nftType, string memory nftRank) external onlyOwner {
        _removeNftTypeAndRank(nftType, nftRank);
    }

    /**
     * @dev Owner updates the minimal staking amount required for users to add themselves onto the HoprNetworkRegistry
     * @param newThreshold Minimum stake of HOPR token
     */
    function ownerUpdateThreshold(uint256 newThreshold) external onlyOwner {
        if (stakeThreshold == newThreshold) revert SameStakingThreshold();
        stakeThreshold = newThreshold;
        emit ThresholdUpdated(stakeThreshold);
    }

    /**
     * @dev update linked stake contract
     * @param _stakeContract address of the staking contract from which registration info is obtained.
     */
    function updateStakeContract(address _stakeContract) external onlyOwner {
        _updateStakeContract(_stakeContract);
    }

    /**
     * @dev adds NFT type and rank to the list of special NFTs.
     * @param nftType Type index of the special HoprBoost NFT
     * @param nftRank HoprBoost rank which is associated to the special NFT, in string
     * @param maxRegistration maximum registration of HOPR node per special NFT
     */
    function _addSpecialNftTypeAndRank(uint256 nftType, string memory nftRank, uint256 maxRegistration) private {
        uint256 i = 0;
        for (i; i < specialNftTypeAndRank.length; i++) {
            // walk through all the types
            if (
                specialNftTypeAndRank[i].nftType == nftType
                    && keccak256(bytes(specialNftTypeAndRank[i].nftRank)) == keccak256(bytes(nftRank))
            ) {
                // already exist, overwrite maxRegistration
                maxRegistrationsPerSpecialNft[i] = maxRegistration;
                emit SpecialNftTypeAndRankAdded(nftType, nftRank, maxRegistration);
                return;
            }
        }
        // did not exist, add to the list
        specialNftTypeAndRank.push(NftTypeAndRank({ nftType: nftType, nftRank: nftRank }));
        maxRegistrationsPerSpecialNft.push(maxRegistration);
        emit SpecialNftTypeAndRankAdded(nftType, nftRank, maxRegistration);
    }

    /**
     * @dev Remove one matching record from list of special NFTs
     * @param nftType Type index of the special HoprBoost NFT
     * @param nftRank HoprBoost rank which is associated to the special NFT, in string
     */
    function _removeSpecialNftTypeAndRank(uint256 nftType, string memory nftRank) private {
        // walk through
        for (uint256 i = 0; i < specialNftTypeAndRank.length; i++) {
            if (
                specialNftTypeAndRank[i].nftType == nftType
                    && keccak256(bytes(specialNftTypeAndRank[i].nftRank)) == keccak256(bytes(nftRank))
            ) {
                // overwrite with the last element in the array
                specialNftTypeAndRank[i] = specialNftTypeAndRank[specialNftTypeAndRank.length - 1];
                specialNftTypeAndRank.pop();
                maxRegistrationsPerSpecialNft[i] =
                    maxRegistrationsPerSpecialNft[maxRegistrationsPerSpecialNft.length - 1];
                maxRegistrationsPerSpecialNft.pop();
                emit SpecialNftTypeAndRankRemoved(nftType, nftRank);
                return;
            }
        }
    }

    /**
     * @dev adds NFT type and rank to the list of eligibles NFTs.
     * @param nftType Type index of the eligible HoprBoost NFT
     * @param nftRank HoprBoost rank which is associated to the eligible NFT, in string
     */
    function _addNftTypeAndRank(uint256 nftType, string memory nftRank) private {
        for (uint256 i = 0; i < eligibleNftTypeAndRank.length; i++) {
            // walk through all the types
            if (
                eligibleNftTypeAndRank[i].nftType == nftType
                    && keccak256(bytes(eligibleNftTypeAndRank[i].nftRank)) == keccak256(bytes(nftRank))
            ) {
                // already exist;
                return;
            }
        }
        eligibleNftTypeAndRank.push(NftTypeAndRank({ nftType: nftType, nftRank: nftRank }));
        emit NftTypeAndRankAdded(nftType, nftRank);
    }

    /**
     * @dev Remove one matching record from list of eligible NFTs
     * @param nftType Type index of the eligible HoprBoost NFT
     * @param nftRank HoprBoost rank which is associated to the eligible NFT, in string
     */
    function _removeNftTypeAndRank(uint256 nftType, string memory nftRank) private {
        // walk through
        for (uint256 i = 0; i < eligibleNftTypeAndRank.length; i++) {
            if (
                eligibleNftTypeAndRank[i].nftType == nftType
                    && keccak256(bytes(eligibleNftTypeAndRank[i].nftRank)) == keccak256(bytes(nftRank))
            ) {
                // overwrite with the last element in the array
                eligibleNftTypeAndRank[i] = eligibleNftTypeAndRank[eligibleNftTypeAndRank.length - 1];
                eligibleNftTypeAndRank.pop();
                emit NftTypeAndRankRemoved(nftType, nftRank);
                return;
            }
        }
    }

    /**
     * Update stake contract address
     * @param _stakeContract address of the staking contract
     */
    function _updateStakeContract(address _stakeContract) private {
        stakeContract = IHoprStake(_stakeContract);
        emit StakeContractUpdated(_stakeContract);
    }
}
