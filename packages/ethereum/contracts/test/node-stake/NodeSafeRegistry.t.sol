// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import { HoprNodeSafeRegistry, HoprNodeSafeRegistryEvents } from "../../src/node-stake/NodeSafeRegistry.sol";
import { PrecompileUtils } from "../utils/Precompiles.sol";
import { ECDSA } from "openzeppelin-contracts/utils/cryptography/ECDSA.sol";
import { Address } from "openzeppelin-contracts/utils/Address.sol";
import { Test } from "forge-std/Test.sol";

// proxy contract to manipulate storage
contract MyNodeSafeRegistry is HoprNodeSafeRegistry {
    constructor() HoprNodeSafeRegistry() { }

    // Only for testing
    function _storeSafeAddress(address nodeAddress, address safeAddress) public {
        HoprNodeSafeRegistry.NodeSafeRecord storage record = _nodeToSafe[nodeAddress];
        record.safeAddress = safeAddress;
    }
}

contract HoprNodeSafeRegistryTest is Test, HoprNodeSafeRegistryEvents {
    address public safe;
    MyNodeSafeRegistry public nodeSafeRegistry;
    address private constant SENTINEL_MODULES = address(0x1);
    uint256 private constant pageSize = 100;

    function setUp() public {
        safe = vm.addr(101); // make address(101) a caller
        nodeSafeRegistry = new MyNodeSafeRegistry();
    }

    /**
     * @dev node can actively register a node
     */
    function testFuzz_RegisterSafeByNode(address safeAddress, address nodeAddress) public {
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0)
                && safeAddress != address(this) && safeAddress != address(nodeSafeRegistry) && safeAddress != vm.addr(303)
        );
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0)
                && nodeAddress != address(this) && nodeAddress != address(nodeSafeRegistry) && nodeAddress != vm.addr(303)
        );
        vm.assume(safeAddress != nodeAddress);

        _helperMockSafe(safeAddress, nodeAddress, true, true);
        vm.prank(nodeAddress);
        vm.expectEmit(true, true, false, false, address(nodeSafeRegistry));
        emit RegisteredNodeSafe(safeAddress, nodeAddress);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev any account can register a safe-node pair with valid signature
     */
    function testFuzz_RegisterSafeWithNodeSig(uint256 nodePrivateKey, address safeAddress) public {
        nodePrivateKey = bound(nodePrivateKey, 1, 1e36);
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0)
                && safeAddress != address(this) && safeAddress != address(nodeSafeRegistry) && safeAddress != vm.addr(303)
        );

        address nodeChainKeyAddress = vm.addr(nodePrivateKey);

        // verify the registration is not known beforehand
        assertFalse(nodeSafeRegistry.isNodeSafeRegistered(safeAddress, nodeChainKeyAddress));

        uint256 nodeSigNonce = nodeSafeRegistry.nodeSigNonce(nodeChainKeyAddress);
        (address nodeAddress, bytes memory sig) =
            _helperBuildSig(nodePrivateKey, safeAddress, nodeChainKeyAddress, nodeSigNonce);

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        vm.expectEmit(true, true, false, false, address(nodeSafeRegistry));
        emit RegisteredNodeSafe(safeAddress, nodeAddress);
        nodeSafeRegistry.registerSafeWithNodeSig(safeAddress, nodeChainKeyAddress, sig);

        // verify the registration worked
        assertTrue(nodeSafeRegistry.isNodeSafeRegistered(safeAddress, nodeChainKeyAddress));

        vm.clearMockedCalls();
    }

    /**
     * @dev signature cannot be re-used
     */
    function testRevert_RegisterSafeWithNodeSigNonceReused(uint256 nodePrivateKey, address safeAddress) public {
        nodePrivateKey = bound(nodePrivateKey, 1, 1e36);
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0)
                && safeAddress != address(this) && safeAddress != address(nodeSafeRegistry) && safeAddress != vm.addr(303)
        );

        address nodeChainKeyAddress = vm.addr(nodePrivateKey);

        // verify the registration is not known beforehand
        assertFalse(nodeSafeRegistry.isNodeSafeRegistered(safeAddress, nodeChainKeyAddress));

        uint256 nodeSigNonce = nodeSafeRegistry.nodeSigNonce(nodeChainKeyAddress);
        (address nodeAddress, bytes memory sig) =
            _helperBuildSig(nodePrivateKey, safeAddress, nodeChainKeyAddress, nodeSigNonce);

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        if (Address.isContract(nodeChainKeyAddress)) {
            vm.expectRevert(HoprNodeSafeRegistry.NodeIsContract.selector);
        }
        nodeSafeRegistry.registerSafeWithNodeSig(safeAddress, nodeChainKeyAddress, sig);

        // fail to re-use the signature
        if (Address.isContract(nodeChainKeyAddress)) {
            vm.expectRevert(HoprNodeSafeRegistry.NodeIsContract.selector);
        } else {
            vm.expectRevert(HoprNodeSafeRegistry.NotValidSignatureFromNode.selector);
        }
        nodeSafeRegistry.registerSafeWithNodeSig(safeAddress, nodeChainKeyAddress, sig);

        vm.clearMockedCalls();
    }

    /**
     * @dev node fail to register a node due to it's registered
     */
    function testRevert_FailToRegisterSafeByNodeDueToRegistered(address safeAddress, address nodeAddress) public {
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0)
                && safeAddress != address(this) && safeAddress != address(nodeSafeRegistry) && safeAddress != vm.addr(303)
        );
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0)
                && nodeAddress != address(this) && nodeAddress != address(nodeSafeRegistry) && nodeAddress != vm.addr(303)
        );
        vm.assume(safeAddress != nodeAddress);

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        nodeSafeRegistry._storeSafeAddress(nodeAddress, safeAddress);

        vm.prank(nodeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.NodeHasSafe.selector);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev node fail to register a node due to the provided safe address is zero
     */
    function testRevert_FailToRegisterSafeByNodeDueToSafeAddressZero(address nodeAddress) public {
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0)
                && nodeAddress != address(this) && nodeAddress != address(nodeSafeRegistry) && nodeAddress != vm.addr(303)
        );

        address safeAddress = address(0);
        _helperMockSafe(safeAddress, nodeAddress, true, true);

        nodeSafeRegistry._storeSafeAddress(nodeAddress, address(1));

        vm.prank(nodeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.SafeAddressZero.selector);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev node fail to register a node due to the provided node address is zero
     */
    function testRevert_FailToRegisterSafeByNodeDueToNodeAddressZero(address safeAddress) public {
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0)
                && safeAddress != address(this) && safeAddress != address(nodeSafeRegistry) && safeAddress != vm.addr(303)
        );

        address nodeAddress = address(0);
        _helperMockSafe(safeAddress, nodeAddress, true, true);

        nodeSafeRegistry._storeSafeAddress(nodeAddress, address(1));

        vm.prank(nodeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.NodeAddressZero.selector);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev node fail to register a node due to the provided node address is a contract
     */
    function testRevert_FailToRegisterSafeByNodeDueToNodeIsContract(address safeAddress, address nodeAddress) public {
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0)
                && safeAddress != address(this) && safeAddress != address(nodeSafeRegistry) && safeAddress != vm.addr(303)
        );
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0)
                && nodeAddress != address(this) && nodeAddress != address(nodeSafeRegistry) && nodeAddress != vm.addr(303)
        );

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        // mock code at nodeAddress
        vm.etch(nodeAddress, hex"00010203040506070809");

        vm.prank(nodeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.NodeIsContract.selector);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
        vm.etch(nodeAddress, hex"");
    }

    /**
     * @dev node fail to register a node due to node and safe addresses are random
     */
    function testRevert_FailToRegisterSafeByNodeDueToNodeNotModuleMember(
        address safeAddress,
        address nodeAddress
    )
        public
    {
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0)
                && safeAddress != address(this) && safeAddress != address(nodeSafeRegistry) && safeAddress != vm.addr(303)
        );
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0)
                && nodeAddress != address(this) && nodeAddress != address(nodeSafeRegistry) && nodeAddress != vm.addr(303)
        );
        vm.assume(safeAddress != nodeAddress);

        _helperMockSafe(safeAddress, nodeAddress, false, false);

        nodeSafeRegistry._storeSafeAddress(nodeAddress, address(0));

        vm.prank(nodeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.NodeNotModuleMember.selector);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev safe can deregister a node by the safe
     */
    function testFuzz_DeregisterNodeBySafe(address safeAddress, address nodeAddress) public {
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0)
                && safeAddress != address(this) && safeAddress != address(nodeSafeRegistry) && safeAddress != vm.addr(303)
        );
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0)
                && nodeAddress != address(this) && nodeAddress != address(nodeSafeRegistry) && nodeAddress != vm.addr(303)
        );
        vm.assume(safeAddress != nodeAddress);

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        vm.prank(nodeAddress);
        nodeSafeRegistry.registerSafeByNode(safeAddress);

        vm.prank(safeAddress);
        vm.expectEmit(true, true, false, false, address(nodeSafeRegistry));
        emit DergisteredNodeSafe(safeAddress, nodeAddress);
        nodeSafeRegistry.deregisterNodeBySafe(nodeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev cannot deregister a random address
     */
    function testRevert_DeregisterNodeBySafeDueToNotValidSafe(address safeAddress, address nodeAddress) public {
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0)
                && safeAddress != address(this) && safeAddress != address(nodeSafeRegistry) && safeAddress != vm.addr(303)
        );
        vm.assume(
            !PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0)
                && nodeAddress != address(this) && nodeAddress != address(nodeSafeRegistry) && nodeAddress != vm.addr(303)
        );
        vm.assume(safeAddress != nodeAddress);

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        vm.prank(nodeAddress);
        nodeSafeRegistry.registerSafeByNode(safeAddress);

        nodeSafeRegistry._storeSafeAddress(nodeAddress, address(1));

        vm.prank(safeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.NotValidSafe.selector);
        nodeSafeRegistry.deregisterNodeBySafe(nodeAddress);

        vm.clearMockedCalls();
    }

    function testFuzz_DomainSeparator(uint256 newChainId) public {
        newChainId = bound(newChainId, 1, 1e18);
        vm.assume(newChainId != block.chainid);
        bytes32 domainSeparatorOnDeployment = nodeSafeRegistry.domainSeparator();

        // call updateDomainSeparator when chainid is the same
        nodeSafeRegistry.updateDomainSeparator();
        assertEq(nodeSafeRegistry.domainSeparator(), domainSeparatorOnDeployment);

        // call updateDomainSeparator when chainid is different
        vm.chainId(newChainId);
        vm.expectEmit(true, true, false, false, address(nodeSafeRegistry));
        emit DomainSeparatorUpdated(
            keccak256(
                abi.encode(
                    keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                    keccak256(bytes("NodeSafeRegistry")),
                    keccak256(bytes(nodeSafeRegistry.VERSION())),
                    newChainId,
                    address(nodeSafeRegistry)
                )
            )
        );
        nodeSafeRegistry.updateDomainSeparator();
        assertTrue(nodeSafeRegistry.domainSeparator() != domainSeparatorOnDeployment);
    }

    // ================== helper functions ==================
    /**
     * @dev mock return of module
     */
    function _helperMockSafe(address safeAddress, address nodeAddress, bool isModuleSet, bool isNodeIncluded) private {
        // modules
        address[] memory modules = new address[](1);
        modules[0] = vm.addr(303);

        vm.mockCall(
            safeAddress,
            abi.encodeWithSignature("getModulesPaginated(address,uint256)", SENTINEL_MODULES, pageSize),
            abi.encode(modules, SENTINEL_MODULES)
        );

        vm.mockCall(modules[0], abi.encodeWithSignature("isHoprNodeManagementModule()"), abi.encode(isModuleSet));

        vm.mockCall(modules[0], abi.encodeWithSignature("isNode(address)", nodeAddress), abi.encode(isNodeIncluded));
    }

    /**
     * @dev Build a registration signature for node
     */
    function _helperBuildSig(
        uint256 mockNodePrivateKey,
        address safeAddress,
        address nodeChainKeyAddress,
        uint256 nonce
    )
        private
        returns (address, bytes memory)
    {
        HoprNodeSafeRegistry.NodeSafeNonce memory nodeSafeNonce = HoprNodeSafeRegistry.NodeSafeNonce({
            safeAddress: safeAddress,
            nodeChainKeyAddress: nodeChainKeyAddress,
            nodeSigNonce: nonce
        });
        bytes32 hashStruct = keccak256(abi.encode(nodeSafeRegistry.NODE_SAFE_TYPEHASH(), nodeSafeNonce));
        // build typed digest
        bytes32 registerHash =
            keccak256(abi.encodePacked(bytes1(0x19), bytes1(0x01), nodeSafeRegistry.domainSeparator(), hashStruct));

        address nodeAddress = vm.addr(mockNodePrivateKey);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(mockNodePrivateKey, registerHash);
        bytes memory sig = abi.encodePacked(r, s, v);

        (address recovered, ECDSA.RecoverError recoverError) = ECDSA.tryRecover(registerHash, sig);
        assertTrue(recoverError == ECDSA.RecoverError.NoError);
        assertEq(recovered, nodeAddress);

        return (nodeAddress, sig);
    }
}
