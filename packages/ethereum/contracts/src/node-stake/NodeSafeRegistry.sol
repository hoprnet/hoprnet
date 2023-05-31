// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

import '@openzeppelin/contracts/utils/cryptography/ECDSA.sol';
import './ISafe.sol';
import './IDualRoleAccess.sol';

error NodeHasSafe();
error NotValidSafe();
error NotValidSignatureFromNode();
error InvalidSafeAddress();
error NotValidSafeOwner();
error NotValidGuardRole();

/**
 * @title Node safe must prove that the Safe is the only authorized controller of
 * the CHAIN_KEY address. This link between the Safe and node's chain-key address
 * should be registered upon successful verification
 * TODO: This contract can be used by HoprChannels contract or HoprNetworkRegistry
 */
contract HoprNodeSafeRegistry {
  struct NodeSafe {
    address safeAddress;
    address nodeChainKeyAddress;
  }

  // Currently deployed version, starting with 1.0.0
  string private constant version = '1.0.0';

  bytes32 private immutable domainSeparator;
  mapping(address => address) public nodeToSafe;
  // NodeSafe struct type hash.
  // keccak256("NodeSafe(address safeAddress,address nodeChainKeyAddress)");
  bytes32 private constant NODE_SAFE_TYPEHASH = hex'6e9a9ee91e0fce141f0eeaf47e1bfe3af5b5f40e5baf2a86acc37a075199c16d';

  event RegisteredNodeSafe(address indexed safeAddress, address indexed nodeAddress);
  event DergisteredNodeSafe(address indexed safeAddress, address indexed nodeAddress);

  constructor() {
    // following encoding guidelines of EIP712
    domainSeparator = keccak256(
      abi.encode(
        keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)'),
        keccak256(bytes('NodeStakeRegistry')),
        keccak256(bytes(version)),
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
    bytes32 hashStruct = keccak256(abi.encode(NODE_SAFE_TYPEHASH, nodeSafe.safeAddress, nodeSafe.nodeChainKeyAddress));

    // build typed digest
    bytes32 registerHash = keccak256(abi.encode(bytes1(0x19), bytes1(0x01), domainSeparator, hashStruct));

    // verify that signatures is from nodeChainKeyAddress. This signature can only be
    (address recovered, ECDSA.RecoverError error) = ECDSA.tryRecover(registerHash, sig);
    if (error != ECDSA.RecoverError.NoError || recovered != nodeSafe.nodeChainKeyAddress) {
      revert NotValidSignatureFromNode();
    }

    // store those state, emit events etc.
    addNodeSafe(nodeSafe.safeAddress, nodeSafe.nodeChainKeyAddress);
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
    ensureNodeIsOwnerAndHasRole(msg.sender, nodeAddr);

    // update and emit event
    nodeToSafe[nodeAddr] = address(0);
    emit DergisteredNodeSafe(address(0), nodeAddr);
  }

  /**
   * @dev register the Safe by the node, directly with call made by the node
   */
  function registerSafeByNode(address safeAddr) external {
    addNodeSafe(safeAddr, msg.sender);
  }

  /**
   * @dev internal funciton to store safe-node pair and emit events
   */
  function addNodeSafe(address safeAddr, address nodeAddr) internal {
    // check this node hasn't been registered ower
    if (nodeToSafe[nodeAddr] != address(0)) {
      revert NodeHasSafe();
    }
    // Safe address cannot be zero
    if (safeAddr == address(0)) {
      revert InvalidSafeAddress();
    }

    // ensure that node is an owner
    ensureNodeIsOwnerAndHasRole(safeAddr, nodeAddr);

    // update and emit event
    nodeToSafe[nodeAddr] = safeAddr;
    emit RegisteredNodeSafe(safeAddr, nodeAddr);
  }

  /**
   * @dev Ensure that the node address is an owner of safe address
   * Ensure that the node address has the NODE_ROLE on the guard of the safe
   * @param safeAddr address of the Safe
   * @param nodeAddr address of the node
   */
  function ensureNodeIsOwnerAndHasRole(address safeAddr, address nodeAddr) internal view {
    // check safeAddress has nodeChainKeyAddress as owner
    address[] memory owners = ISafe(safeAddr).getOwners();
    uint256 index = 0;
    for (index; index < owners.length; index++) {
      if (owners[index] == nodeAddr) break;
    }
    if (index >= owners.length) {
      revert NotValidSafeOwner();
    }

    // check nodeChainKeyAddress has NODE_ROLE in safeAddress guard
    IDualRoleAccess safeGuard = IDualRoleAccess(ISafe(safeAddr).getGuard());
    if (safeGuard.hasRole(safeGuard.NODE_ROLE(), nodeAddr)) {
      revert NotValidGuardRole();
    }
  }
}
