// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import { AccessControlEnumerable } from "openzeppelin-contracts/access/AccessControlEnumerable.sol";
import { IHoprNetworkRegistryRequirement } from "./interfaces/INetworkRegistryRequirement.sol";

abstract contract HoprNetworkRegistryEvents {
    event NetworkRegistryStatusUpdated(bool indexed isEnabled); // Global toggle of the network registry
    event RequirementUpdated(address indexed requirementImplementation); // Emit when the network registry proxy is
        // updated
    event Registered(address indexed stakingAccount, address indexed nodeAddress); // Emit when a node is included in
        // the registry
    event Deregistered(address indexed stakingAccount, address indexed nodeAddress); // Emit when a node is removed from
        // the registry
    event RegisteredByManager(address indexed stakingAccount, address indexed nodeAddress); // Emit when the contract
        // owner register a node for an account
    event DeregisteredByManager(address indexed stakingAccount, address indexed nodeAddress); // Emit when the contract
        // owner removes a node from the registry
    event EligibilityUpdated(address indexed stakingAccount, bool indexed eligibility); // Emit when the eligibility of
        // an account is updated
}

/**
 *    &&&&
 *    &&&&
 *    &&&&
 *    &&&&  &&&&&&&&&       &&&&&&&&&&&&          &&&&&&&&&&/   &&&&.&&&&&&&&&
 *    &&&&&&&&&   &&&&&   &&&&&&     &&&&&,     &&&&&    &&&&&  &&&&&&&&   &&&&
 *     &&&&&&      &&&&  &&&&#         &&&&   &&&&&       &&&&& &&&&&&     &&&&&
 *     &&&&&       &&&&/ &&&&           &&&& #&&&&        &&&&  &&&&&
 *     &&&&         &&&& &&&&&         &&&&  &&&&        &&&&&  &&&&&
 *     %%%%        /%%%%   %%%%%%   %%%%%%   %%%%  %%%%%%%%%    %%%%%
 *    %%%%%        %%%%      %%%%%%%%%%%    %%%%   %%%%%%       %%%%
 *                                          %%%%
 *                                          %%%%
 *                                          %%%%
 *
 * @title HoprNetworkRegistry
 * @dev Smart contract that maintains a list of hopr node address that are allowed
 * to enter HOPR network.
 * Eligibility of whether a node address can be registered is checked through
 * `IHoprNetworkRegistryRequirement`.
 *
 * When reaching its limits, accounts can remove registered node addresses (`deregister`)
 * before adding more.
 * Eligibility is always checked at registration/deregistration. If there's any update in
 * eligibility, it should be done after deregistration
 *
 * This network registry can be globally enabled/disabled by the manager
 *
 * Implementation of `IHoprNetworkRegistryRequirement` can also be dynamically updated by the
 * manager. Some sample implementations can be found under../proxy/ folder
 *
 * Manager has the power to overwrite the registration
 */
contract HoprNetworkRegistry is AccessControlEnumerable, HoprNetworkRegistryEvents {
    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");

    // Implementation of network registry proxy
    IHoprNetworkRegistryRequirement public requirementImplementation;
    // counter for registered nodes per staking account
    mapping(address => uint256) public countRegisterdNodesPerAccount;
    // account address that a node is registered to
    mapping(address => address) public nodeRegisterdToAccount;
    bool public enabled;

    error GloballyDisabledRegistry();
    error GloballyEnabledRegistry();
    error NodeAlreadyRegisterd(address nodeAddress);
    error NodeNotYetRegisterd(address nodeAddress);
    error NodeRegisterdToOtherAccount(address nodeAddress);
    error NotEnoughAllowanceToRegisterNode();
    error CannotOperateForNode(address nodeAddress);
    error ArrayLengthNotMatch();

    /**
     * @dev Network registry can be globally toggled. If `enabled === true`, only nodes registered
     * in this contract with an eligible staking account associated can join HOPR network; If `!enabled`,
     * all the nodes can join HOPR network.
     */
    modifier mustBeEnabled() {
        if (!enabled) {
            revert GloballyDisabledRegistry();
        }
        _;
    }

    /**
     * @dev Specify NetworkRegistry logic implementation and set up manager role
     * enable the network registry on deployment.
     * @param _requirementImplementation address of the network registry logic implementation
     * @param _newOwner address of the contract owner
     * @param _manager address of an additional manager
     */
    constructor(address _requirementImplementation, address _newOwner, address _manager) {
        _setupRole(DEFAULT_ADMIN_ROLE, _newOwner);
        _setupRole(MANAGER_ROLE, _newOwner);
        _setupRole(MANAGER_ROLE, _manager);

        requirementImplementation = IHoprNetworkRegistryRequirement(_requirementImplementation);
        enabled = true;
        emit RequirementUpdated(_requirementImplementation);
        emit NetworkRegistryStatusUpdated(true);
    }

    /**
     * Specify NetworkRegistry logic implementation by a manager
     * @param _requirementImplementation address of the network registry logic implementation
     */
    function updateRequirementImplementation(address _requirementImplementation) external onlyRole(MANAGER_ROLE) {
        requirementImplementation = IHoprNetworkRegistryRequirement(_requirementImplementation);
        emit RequirementUpdated(_requirementImplementation);
    }

    /**
     * Enable globally the network registry by a manager
     */
    function enableRegistry() external onlyRole(MANAGER_ROLE) {
        if (enabled) {
            revert GloballyEnabledRegistry();
        }
        enabled = true;
        emit NetworkRegistryStatusUpdated(true);
    }

    /**
     * Disable globally the network registry by a manager
     */
    function disableRegistry() external onlyRole(MANAGER_ROLE) mustBeEnabled {
        enabled = false;
        emit NetworkRegistryStatusUpdated(false);
    }

    /**
     * @dev Register new nodes by its staking account
     * @notice Transaction will fail, if a node has already been registered
     * registered nodes are ignored by this function
     * Function can only be called when the registry is enabled.
     * @param nodeAddresses addresses nodes to be registered
     */
    function selfRegister(address[] calldata nodeAddresses) external mustBeEnabled {
        for (uint256 i = 0; i < nodeAddresses.length; i++) {
            address nodeAddress = nodeAddresses[i];
            address registeredAccount = nodeRegisterdToAccount[nodeAddress];

            _operationGuard(msg.sender, nodeAddress);

            // register when no record stored
            if (registeredAccount == address(0)) {
                // if node is not registered, update the counter
                countRegisterdNodesPerAccount[msg.sender]++;
                nodeRegisterdToAccount[nodeAddress] = msg.sender;
                emit Registered(msg.sender, nodeAddress);
            } else if (registeredAccount != msg.sender) {
                // if the node was not registered under the caller, revert
                revert NodeAlreadyRegisterd(nodeAddress);
            }
            // skip if the node is already registered
        }

        // check eligibility of the sender staking account
        if (!_checkEligibility(msg.sender)) {
            revert NotEnoughAllowanceToRegisterNode();
        }
        emit EligibilityUpdated(msg.sender, true);
    }

    /**
     * @dev Allows the staking account to deregister a registered node address
     * Function can only be called when the registry is enabled.
     *
     * @param nodeAddresses addresses nodes to be deregistered
     */
    function selfDeregister(address[] calldata nodeAddresses) external mustBeEnabled {
        for (uint256 i = 0; i < nodeAddresses.length; i++) {
            address nodeAddress = nodeAddresses[i];
            address registeredAccount = nodeRegisterdToAccount[nodeAddress];

            // revert when the msg.sender cannot operate for the node
            _operationGuard(msg.sender, nodeAddress);

            if (registeredAccount == msg.sender) {
                // deregister from the caller
                countRegisterdNodesPerAccount[msg.sender]--;
                nodeRegisterdToAccount[nodeAddress] = address(0);
                emit Deregistered(msg.sender, nodeAddress);
            } else if (registeredAccount != address(0)) {
                // when the node is not registered under the caller, revert
                revert NodeRegisterdToOtherAccount(nodeAddress);
            }
            // skip if the node is already deregistered
        }
        // update eligibility
        _sync(msg.sender);
    }

    /**
     * @dev sync the eligibility based on the latest criteria.
     * Function can only be called when the registry is enabled.
     */
    function selfSync() external mustBeEnabled {
        _sync(msg.sender);
    }

    /**
     * @dev Manager adds staking accounts and HOPR node addresses to the registration.
     * Function can be called at any time.
     * Allows manager to register HOPR node addresses even if accounts do not fulfill registration requirements.
     * @notice To overwrite existing entries, a deregister must be called beforehand. Otherwise, revert
     * @param stakingAccounts Array of staking accounts
     * @param nodeAddresses Array of hopr nodes address
     */
    function managerRegister(
        address[] calldata stakingAccounts,
        address[] calldata nodeAddresses
    )
        external
        onlyRole(MANAGER_ROLE)
    {
        if (stakingAccounts.length != nodeAddresses.length) {
            revert ArrayLengthNotMatch();
        }

        for (uint256 i = 0; i < stakingAccounts.length; i++) {
            address nodeAddress = nodeAddresses[i];
            address stakingAccount = stakingAccounts[i];

            if (nodeRegisterdToAccount[nodeAddress] != address(0)) {
                revert NodeAlreadyRegisterd(nodeAddress);
            }

            countRegisterdNodesPerAccount[stakingAccount]++;
            nodeRegisterdToAccount[nodeAddress] = stakingAccount;
            emit RegisteredByManager(stakingAccount, nodeAddress);
        }
    }

    /**
     * @dev Manager removes previously added HOPR node address from the registration.
     * Function can be called at any time. Revert when a node addresss is not registered
     * @notice Owner can even remove self-declared entries.
     * @param nodeAddresses Array of hopr nodes address
     */
    function managerDeregister(address[] calldata nodeAddresses) external onlyRole(MANAGER_ROLE) {
        for (uint256 i = 0; i < nodeAddresses.length; i++) {
            address nodeAddress = nodeAddresses[i];
            address registeredAccount = nodeRegisterdToAccount[nodeAddress];

            if (nodeRegisterdToAccount[nodeAddress] == address(0)) {
                revert NodeNotYetRegisterd(nodeAddress);
            }

            countRegisterdNodesPerAccount[registeredAccount]--;
            nodeRegisterdToAccount[nodeAddress] = address(0);
            emit DeregisteredByManager(registeredAccount, nodeAddress);
        }
    }

    /**
     * @dev Manager syncs a list of staking accounts based on the latest criteria.
     * @param stakingAccounts array of staking accounts whose eligibility will get updated
     */
    function managerSync(address[] calldata stakingAccounts) external onlyRole(MANAGER_ROLE) {
        for (uint256 i = 0; i < stakingAccounts.length; i++) {
            _sync(stakingAccounts[i]);
        }
    }

    /**
     * @dev Manager force syncs the eligibility for staking accounts
     * @param stakingAccounts array of staking accounts whose eligibility will get updated
     */
    function managerForceSync(
        address[] calldata stakingAccounts,
        bool[] memory eligibilities
    )
        external
        onlyRole(MANAGER_ROLE)
    {
        if (stakingAccounts.length != eligibilities.length) {
            revert ArrayLengthNotMatch();
        }
        for (uint256 i = 0; i < stakingAccounts.length; i++) {
            if (eligibilities[i]) {
                emit EligibilityUpdated(stakingAccounts[i], true);
            } else {
                // if the account is no longer eligible
                emit EligibilityUpdated(stakingAccounts[i], false);
            }
        }
    }

    /**
     * @dev Returns if a hopr address is registered by a given account.
     * @param nodeAddress hopr node address
     * @param account account address
     */
    function isNodeRegisteredByAccount(address nodeAddress, address account) public view returns (bool) {
        if (nodeAddress == address(0) || account == address(0)) {
            return false;
        }
        return account == nodeRegisterdToAccount[nodeAddress];
    }

    /**
     * @dev Returns if a hopr address is registered and its associated account is eligible or not.
     * @param nodeAddress hopr node address
     */
    function isNodeRegisteredAndEligible(address nodeAddress) public view returns (bool) {
        if (nodeAddress == address(0)) {
            return false;
        }
        // check if peer id is registered
        address registeredAccount = nodeRegisterdToAccount[nodeAddress];
        if (registeredAccount == address(0)) {
            // this address has never been registered
            return false;
        }
        return _checkEligibility(registeredAccount);
    }

    /**
     * @dev Returns if an account address is eligible according to the criteria defined in the proxy implementation
     * It also checks if a node peer id is associated with the account.
     * @param stakingAccount account address that runs hopr node
     */
    function isAccountEligible(address stakingAccount) public view returns (bool) {
        return _checkEligibility(stakingAccount);
    }

    /**
     * @dev Returns the number of nodes that the staking account can self register,
     * if _checkEligibility() check goes through
     * @param stakingAccount address of the staking account
     */
    function maxAdditionalRegistrations(address stakingAccount) public view returns (uint256) {
        uint256 maxAllowedRegistration = requirementImplementation.maxAllowedRegistrations(stakingAccount);
        uint256 registered = countRegisterdNodesPerAccount[stakingAccount];
        if (maxAllowedRegistration > registered) {
            return maxAllowedRegistration - registered;
        } else {
            return 0;
        }
    }

    /**
     * @dev sync the eligibilitybased on the latest criteria.
     * Function can only be called when the registry is enabled.
     * @param stakingAccount address to check its eligibility
     */
    function _sync(address stakingAccount) private {
        // check eligibility of the sender staking account
        if (_checkEligibility(stakingAccount)) {
            emit EligibilityUpdated(stakingAccount, true);
        } else {
            // if the account is no longer eligible
            emit EligibilityUpdated(stakingAccount, false);
        }
    }

    /**
     * @dev given the current registry, check if an account has the number of registered nodes within the limit,
     * which is the eligibility of an account.
     * @notice If an account has registerd more peers than it's currently allowed, the account become ineligible
     * @param stakingAccount address to check its eligibility
     */
    function _checkEligibility(address stakingAccount) private view returns (bool) {
        uint256 maxAllowedRegistration = requirementImplementation.maxAllowedRegistrations(stakingAccount);
        if (maxAllowedRegistration == 0) {
            return false;
        }
        if (countRegisterdNodesPerAccount[stakingAccount] <= maxAllowedRegistration) {
            return true;
        } else {
            return false;
        }
    }

    function _operationGuard(address stakingAccount, address nodeAddress) private view {
        bool isEligibleAction = requirementImplementation.canOperateFor(stakingAccount, nodeAddress);
        // revert when the stakingAccountstakingAccount cannot operate for the node
        if (!isEligibleAction) {
            revert CannotOperateForNode(nodeAddress);
        }
    }
}
