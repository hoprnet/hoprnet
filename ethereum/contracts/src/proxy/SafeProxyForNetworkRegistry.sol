// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.19;

import { IHoprNetworkRegistryRequirement } from "../interfaces/INetworkRegistryRequirement.sol";
import { HoprNodeSafeRegistry } from "../node-stake/NodeSafeRegistry.sol";
import { AccessControlEnumerable } from "openzeppelin-contracts/access/AccessControlEnumerable.sol";

/**
 * @dev Minimum interface for token contract
 */
interface IERC777Snapshot {
    function balanceOfAt(address _owner, uint128 _blockNumber) external view returns (uint256);
}

/**
 * @dev Network Registry Proxy contract, which is compatible with Safe Node Management set up
 * Node with are considered eligible:
 * Nodes' safe holding HOPR token blances above certain threshold
 */
contract HoprSafeProxyForNetworkRegistry is IHoprNetworkRegistryRequirement, AccessControlEnumerable {
    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");
    IERC777Snapshot public token;
    HoprNodeSafeRegistry public nodeSafeRegistry;
    uint256 public stakeThreshold;
    uint128 public snapshotBlockNumber;

    error SameValue();

    event ThresholdUpdated(uint256 indexed threshold);
    event SnapshotUpdated(uint128 indexed blockNumber);
    event TokenAndRegistryUpdated(address indexed token, address indexed nodeSafeRegistry);

    constructor(
        address _owner,
        address _manager,
        uint256 _stakeThreshold,
        uint128 _snapshotBlockNumber,
        address _token,
        address _nodeSafeRegistry
    ) {
        _setupRole(DEFAULT_ADMIN_ROLE, _owner);
        _setupRole(MANAGER_ROLE, _owner);
        _setupRole(MANAGER_ROLE, _manager);
        _updateStakeThreshold(_stakeThreshold);
        _updateSnapshotBlockNumber(_snapshotBlockNumber);

        token = IERC777Snapshot(_token);
        nodeSafeRegistry = HoprNodeSafeRegistry(_nodeSafeRegistry);
        emit TokenAndRegistryUpdated(_token, _nodeSafeRegistry);
    }

    /**
     * @dev See {IERC165-supportsInterface}.
     */
    function supportsInterface(bytes4 interfaceId)
        public
        view
        virtual
        override(AccessControlEnumerable)
        returns (bool)
    {
        return interfaceId == type(IHoprNetworkRegistryRequirement).interfaceId || super.supportsInterface(interfaceId);
    }

    /**
     * @dev Returns the maximum allowed registration
     * check the safe address associated with the node address and compute maxiAllowedRegistration
     * @param safeAddress node address
     */
    function maxAllowedRegistrations(address safeAddress) external view returns (uint256) {
        return token.balanceOfAt(safeAddress, snapshotBlockNumber) / stakeThreshold;
    }

    /**
     * @dev Get if the staking account is eligible to act on node address
     * @param stakingAccount Staking account
     * @param nodeAddress node address
     */
    function canOperateFor(address stakingAccount, address nodeAddress) external view returns (bool eligiblity) {
        return nodeSafeRegistry.nodeToSafe(nodeAddress) == stakingAccount;
    }

    /**
     * @dev Manager updates the block number of the token balance snapshot,
     * which is used for calculating maxAllowedRegistrations
     * @param newSnapshotBlock new block number of the token balance snapshot
     */
    function updateSnapshotBlockNumber(uint128 newSnapshotBlock) external onlyRole(MANAGER_ROLE) {
        if (snapshotBlockNumber == newSnapshotBlock) {
            revert SameValue();
        }
        _updateSnapshotBlockNumber(newSnapshotBlock);
    }

    /**
     * private function to update block number of the token balance snapshot
     * @param _newSnapshotBlock new block number of the token balance snapshot
     */
    function _updateSnapshotBlockNumber(uint128 _newSnapshotBlock) private {
        snapshotBlockNumber = _newSnapshotBlock;
        emit SnapshotUpdated(_newSnapshotBlock);
    }

    /**
     * @dev Manager updates the minimal staking amount required for users to add themselves onto the HoprNetworkRegistry
     * @param newThreshold Minimum stake of HOPR token
     */
    function updateStakeThreshold(uint256 newThreshold) external onlyRole(MANAGER_ROLE) {
        if (stakeThreshold == newThreshold) {
            revert SameValue();
        }
        _updateStakeThreshold(newThreshold);
    }

    /**
     * private function to update the stake threshold
     * @param _newThreshold minimum stake of HOPR token to register a node
     */
    function _updateStakeThreshold(uint256 _newThreshold) private {
        stakeThreshold = _newThreshold;
        emit ThresholdUpdated(_newThreshold);
    }
}
