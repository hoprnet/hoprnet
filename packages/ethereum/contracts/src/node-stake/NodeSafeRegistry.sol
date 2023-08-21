// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

import { ECDSA } from "openzeppelin-contracts/utils/cryptography/ECDSA.sol";
import { IAvatar } from "../interfaces/IAvatar.sol";
import { IHoprNodeManagementModule } from "../interfaces/INodeManagementModule.sol";
import { Address } from "openzeppelin-contracts/utils/Address.sol";

abstract contract HoprNodeSafeRegistryEvents {
    /**
     * Emitted once a safe and node pair gets registered
     */
    event RegisteredNodeSafe(address indexed safeAddress, address indexed nodeAddress);
    /**
     * Emitted once a safe and node pair gets deregistered
     */
    event DergisteredNodeSafe(address indexed safeAddress, address indexed nodeAddress);
    /**
     * Emitted once the domain separator is updated.
     */
    event DomainSeparatorUpdated(bytes32 indexed domainSeparator);
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
 * @title HoprNodeSafeRegistry
 * @dev Node safe must prove that the Safe is the only authorized controller of
 * the CHAIN_KEY address. This link between the Safe and node's chain-key address
 * should be registered upon successful verification
 *
 * The CHAIN_KEY address should not be a contract
 * The Safe addres should be a contract
 * This implies that Safe and CHAIN_KEY address cannot be the same.
 *
 * This contract is meant to be deployed as a standalone contract
 */
contract HoprNodeSafeRegistry is HoprNodeSafeRegistryEvents {
    using Address for address;

    // Node already has mapped to Safe
    error NodeHasSafe();

    // Not a valid Safe address;
    error NotValidSafe();

    // Not a valid signature from node;
    error NotValidSignatureFromNode();

    // Safe address is zero
    error SafeAddressZero();

    // Node address is zero
    error NodeAddressZero();

    // Node address is a contract
    error NodeIsContract();

    // Provided address is not a member of an enabled NodeManagementModule
    error NodeNotModuleMember();

    // Structure to store the mapping between nodes and their associated Safe contracts
    struct NodeSafeRecord {
        address safeAddress;
        uint96 nodeSigNonce;
    }

    // Structure to represent a node-safe pair with a nonce
    struct NodeSafeNonce {
        address safeAddress;
        address nodeChainKeyAddress;
        uint256 nodeSigNonce;
    }

    // Currently deployed version, starting with 1.0.0
    string public constant VERSION = "1.0.0";

    bytes32 public domainSeparator;
    mapping(address => NodeSafeRecord) _nodeToSafe;
    // NodeSafeNonce struct type hash.
    // keccak256("NodeSafeNonce(address safeAddress,address nodeChainKeyAddress,uint256 nodeSigNonce)");
    bytes32 public constant NODE_SAFE_TYPEHASH = hex"a8ac7aed128d1a2da0773fecc80b6265d15f7e62bf4401eb23bd46c3fcf5d2f8";
    // start and end point for linked list of modules
    address private constant SENTINEL_MODULES = address(0x1);
    // page size of querying modules
    uint256 private constant pageSize = 100;

    /**
     * @dev Constructor function to initialize the contract state.
     * Computes the domain separator for EIP-712 verification.
     */
    constructor() {
        // compute the domain separator on deployment
        updateDomainSeparator();
    }

    /**
     * @dev Returns the Safe address associated with a specific node address.
     * @param nodeAddress The address of the Hopr node.
     * @return safeAddress The associated Safe address.
     */
    function nodeToSafe(address nodeAddress) external view returns (address) {
        return _nodeToSafe[nodeAddress].safeAddress;
    }

    /**
     * @dev Returns the nonce of the signature for a specific node address.
     * @param nodeAddress The address of the Hopr node.
     * @return nodeSigNonce The nonce of the node's signature.
     */
    function nodeSigNonce(address nodeAddress) external view returns (uint256) {
        return _nodeToSafe[nodeAddress].nodeSigNonce;
    }

    /**
     * @dev Checks whether a specific node-safe combination is registered.
     * @param safeAddress Address of safe
     * @param nodeChainKeyAddress Address of node
     * @return registered Whether the node-safe combination is registered.
     */
    function isNodeSafeRegistered(address safeAddress, address nodeChainKeyAddress) external view returns (bool) {
        // If node is not registered to any safe, return false
        if (_nodeToSafe[nodeChainKeyAddress].safeAddress == address(0)) {
            return false;
        }

        return _nodeToSafe[nodeChainKeyAddress].safeAddress == safeAddress;
    }

    /**
     * @dev Register the Safe with a signature from the node.
     * This function can be called by any party.
     * @param safeAddress Address of safe
     * @param nodeChainKeyAddress Address of node
     * @param sig The signature provided by the node.
     */
    function registerSafeWithNodeSig(address safeAddress, address nodeChainKeyAddress, bytes calldata sig) external {
        // check adminKeyAddress has added HOPR tokens to the staking contract.

        // Compute the hash of the struct according to EIP712 guidelines
        bytes32 hashStruct = keccak256(
            abi.encode(
                NODE_SAFE_TYPEHASH, safeAddress, nodeChainKeyAddress, _nodeToSafe[nodeChainKeyAddress].nodeSigNonce
            )
        );

        // Build the typed digest for signature verification
        bytes32 registerHash = keccak256(abi.encodePacked(bytes1(0x19), bytes1(0x01), domainSeparator, hashStruct));

        // Verify that the signature is from nodeChainKeyAddress
        (address recovered, ECDSA.RecoverError error) = ECDSA.tryRecover(registerHash, sig);
        if (error != ECDSA.RecoverError.NoError || recovered != nodeChainKeyAddress) {
            revert NotValidSignatureFromNode();
        }

        // store those state, emit events etc.
        addNodeSafe(safeAddress, nodeChainKeyAddress);
    }

    /**
     * @dev Deregisters a Hopr node from its associated Safe and emits relevant events.
     * This function can only be called by the associated Safe.
     * @param nodeAddr The address of the Hopr node to be deregistered.
     */
    function deregisterNodeBySafe(address nodeAddr) external {
        // check this node was registered to the caller
        if (_nodeToSafe[nodeAddr].safeAddress != msg.sender) {
            revert NotValidSafe();
        }

        // Ensure that node is a member of the module
        ensureNodeIsSafeModuleMember(msg.sender, nodeAddr);

        // Update the state and emit the event
        _nodeToSafe[nodeAddr].safeAddress = address(0);
        emit DergisteredNodeSafe(msg.sender, nodeAddr);
    }

    /**
     * @dev Registers a Safe by the node through a direct function call.
     * This function is meant to be called by the Hopr node itself.
     * @param safeAddr The address of the Safe to be registered.
     */
    function registerSafeByNode(address safeAddr) external {
        addNodeSafe(safeAddr, msg.sender);
    }

    /**
     * @dev Recomputes the domain separator in case of a network fork or update.
     * This function should be called by anyone when required.
     * An event is emitted when the domain separator is updated
     */
    function updateDomainSeparator() public {
        // following encoding guidelines of EIP712
        bytes32 newDomainSeparator = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes("NodeSafeRegistry")),
                keccak256(bytes(VERSION)),
                block.chainid,
                address(this)
            )
        );
        if (newDomainSeparator != domainSeparator) {
            domainSeparator = newDomainSeparator;
            emit DomainSeparatorUpdated(domainSeparator);
        }
    }

    /**
     * @dev Internal function to store a node-safe pair and emit relevant events.
     * @param safeAddress Address of safe
     * @param nodeChainKeyAddress Address of node
     */
    function addNodeSafe(address safeAddress, address nodeChainKeyAddress) internal {
        // Safe address cannot be zero
        if (safeAddress == address(0)) {
            revert SafeAddressZero();
        }
        // Safe address cannot be zero
        if (nodeChainKeyAddress == address(0)) {
            revert NodeAddressZero();
        }

        // Ensure that the node address is not a contract address
        if (nodeChainKeyAddress.isContract()) {
            revert NodeIsContract();
        }

        // check this node hasn't been registered ower
        if (_nodeToSafe[nodeChainKeyAddress].safeAddress != address(0)) {
            revert NodeHasSafe();
        }

        // ensure that node is a member of the (enabled) NodeManagementModule
        ensureNodeIsSafeModuleMember(safeAddress, nodeChainKeyAddress);

        NodeSafeRecord storage record = _nodeToSafe[nodeChainKeyAddress];

        // update record
        record.safeAddress = safeAddress;
        record.nodeSigNonce = record.nodeSigNonce + 1; // as of Solidity 0.8, this reverts on overflows

        // update and emit event
        emit RegisteredNodeSafe(safeAddress, nodeChainKeyAddress);
    }

    /**
     * @dev Ensure that the node address is a member of
     * the enabled node management module of the safe
     * @param safeAddress Address of safe
     * @param nodeChainKeyAddress Address of node
     */
    function ensureNodeIsSafeModuleMember(address safeAddress, address nodeChainKeyAddress) internal view {
        // nodeChainKeyAddress must be a member of the enabled node management module
        address nextModule;
        address[] memory modules;
        // there may be many modules, loop through them. Stop at the end point of the linked list
        while (nextModule != SENTINEL_MODULES) {
            // get modules for safe
            (modules, nextModule) = IAvatar(safeAddress).getModulesPaginated(SENTINEL_MODULES, pageSize);
            for (uint256 i = 0; i < modules.length; i++) {
                if (
                    IHoprNodeManagementModule(modules[i]).isHoprNodeManagementModule()
                        && IHoprNodeManagementModule(modules[i]).isNode(nodeChainKeyAddress)
                ) {
                    return;
                }
            }
        }

        // if nodeChainKeyAddress is not a member of a valid HoprNodeManagementModule to the safe, revert
        revert NodeNotModuleMember();
    }
}
