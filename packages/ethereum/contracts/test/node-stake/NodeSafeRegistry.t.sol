// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import {HoprNodeSafeRegistry,HoprNodeSafeRegistryEvents} from "../../src/node-stake/NodeSafeRegistry.sol";
import {PrecompileUtils} from"../utils/Precompiles.sol";
import {ECDSA} from "openzeppelin-contracts/utils/cryptography/ECDSA.sol";
import {Test} from "forge-std/Test.sol";
import {stdStorage, StdStorage} from "forge-std/StdCheats.sol";

contract HoprNodeSafeRegistryTest is Test, HoprNodeSafeRegistryEvents {
    // to alter the storage
    using stdStorage for StdStorage;

    address public safe;
    HoprNodeSafeRegistry public nodeSafeRegistry;
    address private constant SENTINEL_MODULES = address(0x1);
    uint256 private constant pageSize = 100;

    function setUp() public {
        safe = vm.addr(101); // make address(101) a caller
        nodeSafeRegistry = new HoprNodeSafeRegistry();
    }

    /**
     * @dev node can actively register a node
     */
    function testFuzz_RegisterSafeByNode(address safeAddress, address nodeAddress) public {
        vm.assume(!PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0));
        vm.assume(!PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0));

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
        vm.assume(!PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0));

        HoprNodeSafeRegistry.NodeSafe memory nodeSafe =
            HoprNodeSafeRegistry.NodeSafe(safeAddress, vm.addr(nodePrivateKey));

        // verify the registration is not known beforehand
        assertFalse(nodeSafeRegistry.isNodeSafeRegistered(nodeSafe));

        uint256 nodeSigNonce = nodeSafeRegistry.nodeSigNonce(nodeSafe.nodeChainKeyAddress);
        (address nodeAddress, bytes memory sig) = _helperBuildSig(nodePrivateKey, nodeSafe, nodeSigNonce);

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        vm.expectEmit(true, true, false, false, address(nodeSafeRegistry));
        emit RegisteredNodeSafe(safeAddress, nodeAddress);
        nodeSafeRegistry.registerSafeWithNodeSig(nodeSafe, sig);

        // verify the registration worked
        assertTrue(nodeSafeRegistry.isNodeSafeRegistered(nodeSafe));

        vm.clearMockedCalls();
    }

    /**
     * @dev signature cannot be re-used
     */
    function testRevert_RegisterSafeWithNodeSigNonceReused(uint256 nodePrivateKey, address safeAddress) public {
        nodePrivateKey = bound(nodePrivateKey, 1, 1e36);
        vm.assume(!PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0));

        HoprNodeSafeRegistry.NodeSafe memory nodeSafe =
            HoprNodeSafeRegistry.NodeSafe(safeAddress, vm.addr(nodePrivateKey));

        // verify the registration is not known beforehand
        assertFalse(nodeSafeRegistry.isNodeSafeRegistered(nodeSafe));

        uint256 nodeSigNonce = nodeSafeRegistry.nodeSigNonce(nodeSafe.nodeChainKeyAddress);
        (address nodeAddress, bytes memory sig) = _helperBuildSig(nodePrivateKey, nodeSafe, nodeSigNonce);

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        nodeSafeRegistry.registerSafeWithNodeSig(nodeSafe, sig);

        // fail to re-use the signature
        vm.expectRevert(NotValidSignatureFromNode.selector);
        nodeSafeRegistry.registerSafeWithNodeSig(nodeSafe, sig);

        vm.clearMockedCalls();
    }

    /**
     * @dev node fail to register a node due to it's registered
     */
    function testRevert_FailToRegisterSafeByNodeDueToRegistered(address safeAddress, address nodeAddress) public {
        vm.assume(!PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0));
        vm.assume(!PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0));

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        vm.store(
            address(nodeSafeRegistry),
            bytes32(stdstore.target(address(nodeSafeRegistry)).sig("nodeToSafe(address)").with_key(nodeAddress).find()),
            bytes32(abi.encode(address(1)))
        );
        vm.prank(nodeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.NodeHasSafe.selector);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev node fail to register a node due to the provided safe address is zero
     */
    function testRevert_FailToRegisterSafeByNodeDueToSafeAddressZero(address nodeAddress) public {
        vm.assume(!PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0));

        address safeAddress = address(0);
        _helperMockSafe(safeAddress, nodeAddress, true, true);

        // vm.store(
        //     address(nodeSafeRegistry),
        //     bytes32(stdstore.target(address(nodeSafeRegistry)).sig('nodeToSafe(address)').with_key(nodeAddress).find()),
        //     bytes32(abi.encode(address(1)))
        // );
        vm.prank(nodeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.SafeAddressZero.selector);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev node fail to register a node due to the provided node address is zero
     */
    function testRevert_FailToRegisterSafeByNodeDueToNodeAddressZero(address safeAddress) public {
        vm.assume(!PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0));

        address nodeAddress = address(0);
        _helperMockSafe(safeAddress, nodeAddress, true, true);

        // vm.store(
        //     address(nodeSafeRegistry),
        //     bytes32(stdstore.target(address(nodeSafeRegistry)).sig('nodeToSafe(address)').with_key(nodeAddress).find()),
        //     bytes32(abi.encode(address(1)))
        // );
        vm.prank(nodeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.NodeAddressZero.selector);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev node fail to register a node due to node and safe addresses are random
     */
    function testRevert_FailToRegisterSafeByNodeDueToNotSafeOwnerNorNode(address safeAddress, address nodeAddress)
        public
    {
        vm.assume(!PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0));
        vm.assume(!PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0));

        _helperMockSafe(safeAddress, nodeAddress, false, false);

        vm.store(
            address(nodeSafeRegistry),
            bytes32(stdstore.target(address(nodeSafeRegistry)).sig("nodeToSafe(address)").with_key(nodeAddress).find()),
            bytes32(abi.encode(address(0)))
        );
        vm.prank(nodeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.NotSafeOwnerNorNode.selector);
        nodeSafeRegistry.registerSafeByNode(safeAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev safe can deregister a node by the safe
     */
    function testFuzz_DeregisterNodeBySafe(address safeAddress, address nodeAddress) public {
        vm.assume(!PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0));
        vm.assume(!PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0));

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
        vm.assume(!PrecompileUtils.isPrecompileAddress(safeAddress) && safeAddress != address(0));
        vm.assume(!PrecompileUtils.isPrecompileAddress(nodeAddress) && nodeAddress != address(0));

        _helperMockSafe(safeAddress, nodeAddress, true, true);

        vm.prank(nodeAddress);
        nodeSafeRegistry.registerSafeByNode(safeAddress);

        vm.store(
            address(nodeSafeRegistry),
            bytes32(stdstore.target(address(nodeSafeRegistry)).sig("nodeToSafe(address)").with_key(nodeAddress).find()),
            bytes32(abi.encode(address(1)))
        );

        vm.prank(safeAddress);
        vm.expectRevert(HoprNodeSafeRegistry.NotValidSafe.selector);
        nodeSafeRegistry.deregisterNodeBySafe(nodeAddress);

        vm.clearMockedCalls();
    }

    function testFuzz_DomainSeparator(uint256 newChaidId) public {
        newChaidId = bound(newChaidId, 1, 1e18);
        vm.assume(newChaidId != block.chainid);
        bytes32 domainSeparatorOnDeployment = nodeSafeRegistry.domainSeparator();

        // call updateDomainSeparator when chainid is the same
        nodeSafeRegistry.updateDomainSeparator();
        assertEq(nodeSafeRegistry.domainSeparator(), domainSeparatorOnDeployment);

        // call updateDomainSeparator when chainid is different
        vm.chainId(newChaidId);
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
    function _helperBuildSig(uint256 mockNodePrivateKey, HoprNodeSafeRegistry.NodeSafe memory nodeSafe, uint256 nonce)
        private
        returns (address, bytes memory)
    {
        HoprNodeSafeRegistry.NodeSafeNonce memory nodeSafeNonce = HoprNodeSafeRegistry.NodeSafeNonce({
            safeAddress: nodeSafe.safeAddress,
            nodeChainKeyAddress: nodeSafe.nodeChainKeyAddress,
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
