// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import '../../src/node-stake/permissioned-module/NodeManagementModule.sol';
import '../../src/node-stake/permissioned-module/CapabilityPermissions.sol';
import '../../src/node-stake/NodeStakeFactory.sol';
import '../../src/HoprChannels.sol';
import '../../lib/safe-contracts/contracts/Safe.sol';
import "../../script/utils/SafeSuiteLib.sol";
import "../utils/SafeSingleton.sol";
import "../utils/ERC1820Registry.sol";
import "safe-contracts/Safe.sol";
import "safe-contracts/common/Enum.sol";
import 'forge-std/Test.sol';

contract HoprNodeStakeIntegrationTest is Test, ERC1820RegistryFixtureTest, SafeSingletonFixtureTest {
    // to alter the storage
    using stdStorage for StdStorage;

    HoprNodeManagementModule public moduleSingleton;
    HoprNodeStakeFactory public factory;
    HoprChannels public hoprChannels;
    address public hoprToken;

    address public caller;
    address public admin;
    address public module;
    address payable public safe;
    address internal node1;
    address internal node2;
    address internal node3;
    address internal node4;

    /**
    * Manually import events and errors
    */
    event SetMultisendAddress(address multisendAddress);
    event NewHoprNodeStakeModule(address instance);
    event NewHoprNodeStakeSafe(address instance);

    function setUp() public override(ERC1820RegistryFixtureTest, SafeSingletonFixtureTest) {
        super.setUp();
        // deploy safe suites
        deployEntireSafeSuite();

        caller = vm.addr(101); // make make address(101) a caller
        admin = vm.addr(102); // make make address(102) an admin
        moduleSingleton = new HoprNodeManagementModule();
        factory = new HoprNodeStakeFactory();

        // ensure that ERC1820 registry is in place
        vm.etch(ERC1820_REGISTRY_ADDRESS, ERC1820_REGISTRY_DEPLOYED_CODE);
        mustHaveErc1820Registry();

        hoprToken = deployCode("HoprToken.sol:HoprToken");
        hoprChannels = new HoprChannels(hoprToken, 15);
        node1 = 0xFD0E196A4548B9E8Cc28CB94B74B2f79F72C3e09;
        node2 = 0x4FCEB6F2C703E9FE573847EB391964aA58bE7191;
        node3 = 0xFD5328aB5BfCC27BeA70FDeA223Ef7D67Ce30Ef0;
        node4 = 0x62A51865106345B6a63dDf0B3Ef312754338F15D;

        _helperSetupBalanceAndAnnounce();
    }

    /**
     * @dev compare open channel gas cost (through module vs directly with HoprChannels contract)
     */
    function test_OpenChannel() public {
        uint256 safeCreationNonce = 3;

        // create node management safe + module; use 
        uint256 gasStart = gasleft();
        (module, safe) = _helperSetupNodeStaking(safeCreationNonce);
        uint256 gasCheckpoint1 = gasleft();
        emit log_named_uint("Gas Used for setup", gasStart - gasCheckpoint1); 

        vm.startPrank(node1);
        // open a channel with `fundChannelMulti`
        bytes memory fundChannelTx = abi.encodeWithSignature("fundChannelMulti(address,address,uint256,uint256)", node1, node2, 1 ether, 2 ether);
        uint256 gasCheckpoint2 = gasleft();
        HoprNodeManagementModule(module).execTransactionFromModule(address(hoprChannels), 0, fundChannelTx, Enum.Operation.Call);
        uint256 gasCheckpoint3 = gasleft();
        emit log_named_uint("Gas Used for fundChannelMulti via Module", gasCheckpoint2 - gasCheckpoint3); 
        
        (, bytes memory returndataSafeTokenBalance) = hoprToken.staticcall(
            abi.encodeWithSignature('balanceOf(address)', safe)
        );
        assertEq(abi.decode(returndataSafeTokenBalance, (uint256)), 17 ether);
        (, bytes memory returndataChannelsTokenBalance) = hoprToken.staticcall(
            abi.encodeWithSignature('balanceOf(address)',  address(hoprChannels))
        );
        assertEq(abi.decode(returndataChannelsTokenBalance, (uint256)), 3 ether);

        vm.stopPrank();
        vm.startPrank(node3); // stop and start prank = vm.changePrank(node3);

        // open a channel with directly with HoprChannels contract
        hoprToken.call(
            abi.encodeWithSignature("approve(address,uint256)", address(hoprChannels), 10 ether)
        );
        uint256 gasCheckpoint4 = gasleft();
        hoprChannels.fundChannelMulti(node3, node4, 1 ether, 2 ether);
        uint256 gasCheckpoint5 = gasleft();
        emit log_named_uint("Gas Used for fundChannelMulti to Channels", gasCheckpoint4 - gasCheckpoint5); 
        
        (, bytes memory returndataNode3TokenBalance) = hoprToken.staticcall(
            abi.encodeWithSignature('balanceOf(address)', node3)
        );
        assertEq(abi.decode(returndataNode3TokenBalance, (uint256)), 7 ether);
        (, bytes memory returndatChannelsTokenBalance2) = hoprToken.staticcall(
            abi.encodeWithSignature('balanceOf(address)',  address(hoprChannels))
        );
        assertEq(abi.decode(returndatChannelsTokenBalance2, (uint256)), 6 ether);
        vm.stopPrank();
    }

    /**
     * @dev compare bump channel gas cost (through module vs directly with HoprChannels contract)
     */
    function test_BumpChannel() public {
        uint256 safeCreationNonce = 3;
        bytes32 newCommitment = 0x6e6577436f6d6d69746d656e7400000000000000000000000000000000000000;

        // create node management safe + module; use 
        uint256 gasStart = gasleft();
        (module, safe) = _helperSetupNodeStaking(safeCreationNonce);
        vm.startPrank(caller);
        hoprChannels.fundChannelMulti(node2, node1, 1 ether, 2 ether);
        hoprChannels.fundChannelMulti(node3, node4, 1 ether, 2 ether);
        vm.stopPrank();
        uint256 gasCheckpoint1 = gasleft();
        emit log_named_uint("Gas Used for setup", gasStart - gasCheckpoint1); 

        vm.startPrank(node1);
        // bump a channel with `bumpChannel`
        bytes memory bumpChannelTx = abi.encodeWithSignature("bumpChannel(address,address,bytes32)", node2, node1, newCommitment);
        uint256 gasCheckpoint2 = gasleft();
        HoprNodeManagementModule(module).execTransactionFromModule(address(hoprChannels), 0, bumpChannelTx, Enum.Operation.Call);
        uint256 gasCheckpoint3 = gasleft();
        emit log_named_uint("Gas Used for bumpChannel via Module", gasCheckpoint2 - gasCheckpoint3); 
        
        vm.stopPrank();
        vm.startPrank(node4); // stop and start prank = vm.changePrank(node3);

        uint256 gasCheckpoint4 = gasleft();
        hoprChannels.bumpChannel(node3, node4, newCommitment);
        uint256 gasCheckpoint5 = gasleft();
        emit log_named_uint("Gas Used for bumpChannel to Channels", gasCheckpoint4 - gasCheckpoint5); 
        vm.stopPrank();
    }

    /**
     * @dev compare initiate channel closure gas cost (through module vs directly with HoprChannels contract)
     */
    function test_InitiateChannelClosure() public {
        uint256 safeCreationNonce = 3;
        bytes32 newCommitment = 0x6e6577436f6d6d69746d656e7400000000000000000000000000000000000000;

        // create node management safe + module; use 
        uint256 gasStart = gasleft();
        (module, safe) = _helperSetupNodeStaking(safeCreationNonce);
        vm.startPrank(caller);
        hoprChannels.fundChannelMulti(node1, node2, 1 ether, 2 ether);
        hoprChannels.fundChannelMulti(node3, node4, 1 ether, 2 ether);
        vm.stopPrank();
        vm.prank(node2);
        hoprChannels.bumpChannel(node1, node2, newCommitment);
        vm.prank(node4);
        hoprChannels.bumpChannel(node3, node4, newCommitment);
        uint256 gasCheckpoint1 = gasleft();
        emit log_named_uint("Gas Used for setup", gasStart - gasCheckpoint1); 

        vm.startPrank(node1);
        // bump a channel with `bumpChannel`
        bytes memory initiateChannelClosureTx = abi.encodeWithSignature("initiateChannelClosure(address,address)", node1, node2);
        uint256 gasCheckpoint2 = gasleft();
        HoprNodeManagementModule(module).execTransactionFromModule(address(hoprChannels), 0, initiateChannelClosureTx, Enum.Operation.Call);
        uint256 gasCheckpoint3 = gasleft();
        emit log_named_uint("Gas Used for initiateChannelClosure via Module", gasCheckpoint2 - gasCheckpoint3); 
        
        vm.stopPrank();
        vm.startPrank(node3); // stop and start prank = vm.changePrank(node3);

        uint256 gasCheckpoint4 = gasleft();
        hoprChannels.initiateChannelClosure(node3, node4);
        uint256 gasCheckpoint5 = gasleft();
        emit log_named_uint("Gas Used for initiateChannelClosure to Channels", gasCheckpoint4 - gasCheckpoint5); 
        vm.stopPrank();
    }

    /**
     * @dev compare finalize channel closure gas cost (through module vs directly with HoprChannels contract)
     */
    function test_FinalizeChannelClosure() public {
        uint256 safeCreationNonce = 3;
        bytes32 newCommitment = 0x6e6577436f6d6d69746d656e7400000000000000000000000000000000000000;

        // create node management safe + module; use 
        uint256 gasStart = gasleft();
        (module, safe) = _helperSetupNodeStaking(safeCreationNonce);
        vm.startPrank(caller);
        hoprChannels.fundChannelMulti(node1, node2, 1 ether, 2 ether);
        hoprChannels.fundChannelMulti(node3, node4, 1 ether, 2 ether);
        vm.stopPrank();
        vm.prank(node2);
        hoprChannels.bumpChannel(node1, node2, newCommitment);
        vm.prank(node4);
        hoprChannels.bumpChannel(node3, node4, newCommitment);
        vm.prank(node1);
        hoprChannels.initiateChannelClosure(node1, node2);
        vm.prank(node3);
        hoprChannels.initiateChannelClosure(node3, node4);
        uint256 gasCheckpoint1 = gasleft();
        emit log_named_uint("Gas Used for setup", gasStart - gasCheckpoint1); 

        // increase enough time for channel closure;
        vm.warp(block.timestamp + 100);

        vm.startPrank(node1);
        // bump a channel with `bumpChannel`
        bytes memory initiateChannelClosureTx = abi.encodeWithSignature("finalizeChannelClosure(address,address)", node1, node2);
        uint256 gasCheckpoint2 = gasleft();
        HoprNodeManagementModule(module).execTransactionFromModule(address(hoprChannels), 0, initiateChannelClosureTx, Enum.Operation.Call);
        uint256 gasCheckpoint3 = gasleft();
        emit log_named_uint("Gas Used for finalizeChannelClosure via Module", gasCheckpoint2 - gasCheckpoint3); 
        
        vm.stopPrank();
        vm.startPrank(node3); // stop and start prank = vm.changePrank(node3);

        uint256 gasCheckpoint4 = gasleft();
        hoprChannels.finalizeChannelClosure(node3, node4);
        uint256 gasCheckpoint5 = gasleft();
        emit log_named_uint("Gas Used for finalizeChannelClosure to Channels", gasCheckpoint4 - gasCheckpoint5); 
        vm.stopPrank();
    }

    /**
     * @dev Give 1000 HOPR token to admin; 10 HOPR to node3; 99999 HOPR token to caller
     * Announce all the nodes and approve hoprChannels to move funds for node3
     */
    function _helperSetupBalanceAndAnnounce() internal {
        // mock 1000 HOPR token in admin account; 10 HOPR token in node3
        vm.store(
            hoprToken,
            bytes32(stdstore.target(hoprToken).sig('balanceOf(address)').with_key(admin).find()),
            bytes32(abi.encode(1000 ether))
        );
        vm.store(
            hoprToken,
            bytes32(stdstore.target(hoprToken).sig('balanceOf(address)').with_key(node3).find()),
            bytes32(abi.encode(10 ether))
        );
        vm.store(
            hoprToken,
            bytes32(stdstore.target(hoprToken).sig('balanceOf(address)').with_key(caller).find()),
            bytes32(abi.encode(99999 ether))
        );

        // open a channel with directly with HoprChannels contract
        vm.prank(node3);
        hoprToken.call(
            abi.encodeWithSignature("approve(address,uint256)", address(hoprChannels), 10 ether)
        );
        vm.prank(caller);
        hoprToken.call(
            abi.encodeWithSignature("approve(address,uint256)", address(hoprChannels), 99999 ether)
        );

        // all the nodes are announced
        vm.prank(node1);
        hoprChannels.announce(hex"04cd4f9837316baa30b5095d5f2aff80bdbcab7ee611508e729d77d4c84ec328baa5da780a98d32d38606f56a2e3d00ba9783860f521e65f32fe8d3795451c4965", hex"0123");
        vm.prank(node2);
        hoprChannels.announce(hex"041e25d85318df6d4f8c204233bdcfae4149b8110d0d41ab76d644e16e0106673df708e2e1c51fcf90f45879419b4d2fadf1354a64f825dc103caad2f0777ef5e7", hex"0123");
        vm.prank(node3);
        hoprChannels.announce(hex"044c8b89f29d8d26461f0962bf65320dde9cb8559bf0886b78c4879d89e349bdb59a110f6f803786c1fef9780d8d7e0e6696ceae3bba9624e5b30709ca4cf7435d", hex"0123");
        vm.prank(node4);
        hoprChannels.announce(hex"04792f788df2ef7763088a388ebb16d6e03147a5f715d4083b51283db23aeb76bf62272e232106a57fc5304fdea1a0686b2da557cbaec37964790dba86842167cd", hex"0123");
    }

    function _helperSetupNodeStaking(
        uint256 safeCreationNonce
    ) internal returns (
        address module, 
        address payable safe
    ) {
        vm.startPrank(caller);
        // create safe and a module; wire them up
        (module, safe) = factory.clone(address(moduleSingleton), admin, safeCreationNonce);
        
        vm.stopPrank();
        vm.startPrank(admin); // stop and start prank = vm.changePrank(admin);

        // safe admin add a hopr channel contract as target; equivalent to HoprNodeManagementModule(module).addChannelsAndTokenTarget(address(hoprChannels))
        bytes memory addTargetTx = abi.encodeWithSignature("addChannelsAndTokenTarget(address)", address(hoprChannels));
        _helperSendSafeTxFromPrivKey(safe, module, 102, addTargetTx);

        // safe admin transfer some HOPR tokens to the safe
        hoprToken.call(
            abi.encodeWithSignature("transfer(address,uint256)", safe, 20 ether)
        );
        
        // admin add a node to the safe; equivalent to HoprNodeManagementModule(module).addNode(node1);
        bytes memory addNodeTx = abi.encodeWithSignature("addNode(address)", node1);
        _helperSendSafeTxFromPrivKey(safe, module, 102, addNodeTx);

        // by default this node is allowed to operate on behalf of the safe

        // approve some tokens to be transferred from,
        bytes memory approvalTx = abi.encodeWithSignature("approve(address,uint256)", address(hoprChannels), 20 ether);
        _helperSendSafeTxFromPrivKey(safe, hoprToken, 102, approvalTx);
        
        // assertEq(hoprToken.allowance(safe, address(hoprChannels)), 20 ether);
        (, bytes memory returndataAllowance0) = hoprToken.staticcall(
            abi.encodeWithSignature('allowance(address,address)', safe, address(hoprChannels))
        );
        assertEq(abi.decode(returndataAllowance0, (uint256)), 20 ether);
        vm.stopPrank();
    }

    function _helperSendSafeTxFromPrivKey(address payable safe, address to, uint256 privKey, bytes memory payload) internal {
        bytes32 txHash = Safe(safe).getTransactionHash(
            to, 
            0, 
            payload, 
            Enum.Operation.Call, 
            0, 
            0, 
            0, 
            address(0), 
            address(0),
            Safe(safe).nonce()
        );
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(privKey, txHash);
        bytes memory signature = abi.encodePacked(r, s, v);
        
        Safe(safe).execTransaction(
            to, 
            0, 
            payload, 
            Enum.Operation.Call, 
            0, 
            0, 
            0, 
            address(0), 
            payable(address(0)), 
            signature
        );

    }
}
