pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Test.sol";
import "../../src/utils/EnumerableTargetSet.sol";

contract EnumerableTargetSetTest is Test {
    address public targetUtilsLibAddress;

    using TargetUtils for Target;
    Target internal target;

    function setUp() public {
        targetUtilsLibAddress = deployCode("EnumerableTargetSet.sol:TargetUtils");
    }

    function test_GetDefaultPermissionAt(uint256 position) public {
        uint256 limit = TargetUtils.getNumDefaultFunctionPermissions() - 1;
        position = bound(position, 0, limit);
        FunctionPermission permission = TargetUtils.getDefaultFunctionPermissionAt(target, position);
        assertEq(uint8(permission), 0);
    }

    function testRevert_GetDefaultPermissionAt(uint256 offset) public {
        offset = bound(offset, 1, 1e36);
        
        uint256 position = TargetUtils.getNumDefaultFunctionPermissions() + offset;
        vm.expectRevert(FunctionPermissionsTooMany.selector);
        TargetUtils.getDefaultFunctionPermissionAt(target, position);
    }

    function testGetTargetAddress() public {
        assertTrue(target.getTargetAddress() == address(0));
    }
}
