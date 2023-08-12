// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

import {ECDSA} from "openzeppelin-contracts/utils/cryptography/ECDSA.sol";
import {IAvatar} from "../interfaces/IAvatar.sol";
import {IHoprNodeManagementModule} from "../interfaces/INodeManagementModule.sol";

abstract contract HoprNodeSafeRegistryEvents {
    event RegisteredNodeSafe(address indexed safeAddress, address indexed nodeAddress);
    event DergisteredNodeSafe(address indexed safeAddress, address indexed nodeAddress);
}

/**
 * @title Node safe must prove that the Safe is the only authorized controller of
 * the CHAIN_KEY address. This link between the Safe and node's chain-key address
 * should be registered upon successful verification
 * This contract is meant to be deployed as a standalone contract
 */
contract HoprNodeSafeRegistry is HoprNodeSafeRegistryEvents {
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

    // Provided address is neither an owner of Safe nor a member of an enabled NodeManagementModule
    error NotSafeOwnerNorNode();

    struct NodeSafe {
        address safeAddress;
        address nodeChainKeyAddress;
    }

    // Currently deployed version, starting with 1.0.0
    string public constant VERSION = "1.0.0";

    bytes32 public immutable domainSeparator;
    mapping(address => address) public nodeToSafe;
    // NodeSafe struct type hash.
    // keccak256("NodeSafe(address safeAddress,address nodeChainKeyAddress)");
    bytes32 public constant NODE_SAFE_TYPEHASH = hex"6e9a9ee91e0fce141f0eeaf47e1bfe3af5b5f40e5baf2a86acc37a075199c16d";
    // start and end point for linked list of modules
    address private constant SENTINEL_MODULES = address(0x1);
    // page size of querying modules
    uint256 private constant pageSize = 100;

    constructor() {
        // following encoding guidelines of EIP712
        domainSeparator = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes("NodeStakeRegistry")),
                keccak256(bytes(VERSION)),
                block.chainid,
                address(this)
            )
        );
    }

    /**
     * @dev register the Safe with a signature from the node
     * This function can be called by any party
     */
    function registerSafeWithNodeSig(NodeSafe memory nodeSafe, bytes calldata sig) external {
        // check adminKeyAddress has added HOPR tokens to the staking contract.

        // following encoding guidelines of EIP712
        bytes32 hashStruct = keccak256(abi.encode(NODE_SAFE_TYPEHASH, nodeSafe));

        // build typed digest
        bytes32 registerHash = keccak256(abi.encodePacked(bytes1(0x19), bytes1(0x01), domainSeparator, hashStruct));

        // verify that signatures is from nodeChainKeyAddress. This signature can only be
        (address recovered, ECDSA.RecoverError error) = ECDSA.tryRecover(registerHash, sig);
        if (error != ECDSA.RecoverError.NoError || recovered != nodeSafe.nodeChainKeyAddress) {
            revert NotValidSignatureFromNode();
        }

        // store those state, emit events etc.
        addNodeSafe(nodeSafe);
    }

    /**
     * @dev checks whether a NodeSafe combination has been registered before.
     */
    function isNodeSafeRegistered(NodeSafe memory nodeSafe) external view returns (bool) {
        return nodeToSafe[nodeSafe.nodeChainKeyAddress] == nodeSafe.safeAddress;
    }

    /**
     * @dev external funciton to remove safe-node pair and emit events
     * This function can only be called by the Safe
     */
    function deregisterNodeBySafe(address nodeAddr) external {
        // check this node was registered to the caller
        if (nodeToSafe[nodeAddr] != msg.sender) {
            revert NotValidSafe();
        }

        // ensure that node is an owner
        ensureNodeIsSafeModuleMember(NodeSafe({safeAddress: msg.sender, nodeChainKeyAddress: nodeAddr}));

        // update and emit event
        nodeToSafe[nodeAddr] = address(0);
        emit DergisteredNodeSafe(msg.sender, nodeAddr);
    }

    /**
     * @dev register the Safe by the node, directly with call made by the node
     */
    function registerSafeByNode(address safeAddr) external {
        addNodeSafe(NodeSafe({safeAddress: safeAddr, nodeChainKeyAddress: msg.sender}));
    }

    /**
     * @dev internal funciton to store safe-node pair and emit events
     */
    function addNodeSafe(NodeSafe memory nodeSafe) internal {
        // Safe address cannot be zero
        if (nodeSafe.safeAddress == address(0)) {
            revert SafeAddressZero();
        }
        // Safe address cannot be zero
        if (nodeSafe.nodeChainKeyAddress == address(0)) {
            revert NodeAddressZero();
        }

        // check this node hasn't been registered ower
        if (nodeToSafe[nodeSafe.nodeChainKeyAddress] != address(0)) {
            revert NodeHasSafe();
        }

        // ensure that node is either an owner or a member of the (enabled) NodeManagementModule
        ensureNodeIsSafeModuleMember(nodeSafe);

        // update and emit event
        nodeToSafe[nodeSafe.nodeChainKeyAddress] = nodeSafe.safeAddress;
        emit RegisteredNodeSafe(nodeSafe.safeAddress, nodeSafe.nodeChainKeyAddress);
    }

    /**
     * @dev Ensure that the node address is either an owner or a member of
     * the enebled node management module of the safe
     * @param nodeSafe struct to check
     */
    function ensureNodeIsSafeModuleMember(NodeSafe memory nodeSafe) internal view {
        // if nodeChainKeyAddress is not an owner, it must be a member of the enabled node management module
        address nextModule;
        address[] memory modules;
        // there may be many modules, loop through them
        while (nextModule != SENTINEL_MODULES) {
            // get modules for safe
            (modules, nextModule) = IAvatar(nodeSafe.safeAddress).getModulesPaginated(SENTINEL_MODULES, pageSize);
            for (uint256 i = 0; i < modules.length; i++) {
                if (
                    IHoprNodeManagementModule(modules[i]).isHoprNodeManagementModule()
                        && IHoprNodeManagementModule(modules[i]).isNode(nodeSafe.nodeChainKeyAddress)
                ) {
                    return;
                }
            }
        }

        // if nodeChainKeyAddress is neither an owner nor a member of a valid HoprNodeManagementModule to the safe, revert
        revert NotSafeOwnerNorNode();
    }
}
