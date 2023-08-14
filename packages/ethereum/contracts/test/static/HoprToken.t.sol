// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import "../../src/static/HoprToken.sol";
import "../utils/ERC1820Registry.sol";
import "forge-std/Test.sol";

contract HoprTokenTest is Test, ERC1820RegistryFixtureTest {
    HoprToken public hoprToken;
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant DEFAULT_ADMIN_ROLE = 0x00;

    event Minted(address indexed operator, address indexed to, uint256 amount, bytes data, bytes operatorData);

    function setUp() public virtual override {
        super.setUp();
        hoprToken = new HoprToken();
    }

    function testAdminRoleIsGiven() public {
        // vm.prank(accountA.accountAddr);
        assertEq(hoprToken.getRoleMemberCount(DEFAULT_ADMIN_ROLE), 1);
        assertEq(hoprToken.getRoleMemberCount(MINTER_ROLE), 0);
        assertEq(hoprToken.getRoleMember(DEFAULT_ADMIN_ROLE, 0), address(this));
    }

    function testRevert_CannotMintWithoutMinterRole(uint256 amount) public {
        // prank deployer account
        vm.prank(address(this));
        vm.expectRevert("caller does not have minter role");
        hoprToken.mint(vm.addr(1), amount, hex"00", hex"00");
    }

    function testMintWithMinterRole(uint256 amount) public {
        amount = bound(amount, 0, 1e36);
        vm.prank(address(this));
        // give deployer account minter role.
        hoprToken.grantRole(MINTER_ROLE, address(this));
        vm.expectEmit(true, false, false, false, address(hoprToken));
        emit Minted(address(this), vm.addr(1), amount, hex"00", hex"00");
        hoprToken.mint(vm.addr(1), amount, hex"00", hex"00");
    }
}
