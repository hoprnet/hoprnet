// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import '../IHoprNetworkRegistryRequirement.sol';
import 'openzeppelin-contracts-4.8.3/access/AccessControlEnumerable.sol';

/**
 * @dev Minimum interface for NodeSafeRegistry contract
 */
contract INodeSafeRegistry {
  function nodeToSafe(address node) public view returns (address) {}
}

/**
 * @dev Minimum interface for token contract
 */
interface IERC20 {
    function balanceOf(address account) external view returns (uint256);
}

/**
 * @dev Network Registry Proxy contract, which is compatible with Safe Node Management set up
 * Node with are considered eligible:
 * Nodes' safe holding HOPR token blances above certain threshold
 */
contract HoprStakingProxyForNetworkRegistry is IHoprNetworkRegistryRequirement, AccessControlEnumerable {
    bytes32 public constant MANAGER_ROLE = keccak256('MANAGER_ROLE');
    IERC20 public token;
    INodeSafeRegistry public nodeSafeRegistry;
    uint256 public stakeThreshold;

    error SameValue();
    event ThresholdUpdated(uint256 indexed threshold);
    event TokenAndRegistryUpdated(address indexed token, address indexed nodeSafeRegistry);

    constructor(
        address _owner,
        uint256 _stakeThreshold,
        address _token,
        address _nodeSafeRegistry
    ) {
        _setupRole(DEFAULT_ADMIN_ROLE, _owner);
        _setupRole(MANAGER_ROLE, _owner);
        _updateStakeThreshold(_stakeThreshold);

        token = IERC20(_token);
        nodeSafeRegistry = INodeSafeRegistry(_nodeSafeRegistry);
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
     * @param nodeAddress Node address
     */
    function maxAllowedRegistrations(address nodeAddress) external view returns (uint256) {
        address safeAddress = nodeSafeRegistry.nodeToSafe(nodeAddress);
        return token.balanceOf(safeAddress) / stakeThreshold;
    }

    /**
     * @dev Owner updates the minimal staking amount required for users to add themselves onto the HoprNetworkRegistry
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