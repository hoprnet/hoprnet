// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Test.sol";

import { HoprWrapper } from "../../src/static/HoprWrapper.sol";
import { HoprToken } from "../../src/static/HoprToken.sol";
import { ERC1820RegistryFixtureTest } from "../utils/ERC1820Registry.sol";
import { ERC677Mock } from "../../src/static/stake/mocks/ERC677Mock.sol";
import { IERC20 } from "openzeppelin-contracts-4.4.2/token/ERC20/IERC20.sol";

contract HoprWrapperTest is Test, ERC1820RegistryFixtureTest {
    // to alter the storage
    using stdStorage for StdStorage;

    HoprWrapper public hoprWrapper;
    HoprToken public wxHoprToken;
    ERC677Mock public xHoprToken;

    address public newOwner;
    address public holder;
    uint256 public constant INTIAL_BALANCE = 100;

    /**
     * Manually import the errors and events
     */
    event Wrapped(address indexed account, uint256 amount);
    event Unwrapped(address indexed account, uint256 amount);

    function setUp() public virtual override {
        super.setUp();
        newOwner = vm.addr(100); // make address(100) new owner
        holder = vm.addr(1); // make address(1) a holder
        vm.startPrank(newOwner);
        wxHoprToken = new HoprToken();
        xHoprToken = new ERC677Mock();
        hoprWrapper = new HoprWrapper(IERC20(address(xHoprToken)), wxHoprToken);

        // allow wrapper to mint wxHOPR required for swapping
        wxHoprToken.grantRole(wxHoprToken.MINTER_ROLE(), address(hoprWrapper));
        // mint some initial xHOPR for holder
        address[] memory recipients = new address[](1);
        recipients[0] = holder;
        xHoprToken.batchMintInternal(recipients, INTIAL_BALANCE);
        vm.stopPrank();
    }

    /**
     * @dev Wrap xHOPR tokens to wxHOPR tokens
     */
    function testFuzz_WrapSomexHOPR(uint256 amount) public {
        amount = bound(amount, 1, INTIAL_BALANCE);
        vm.prank(holder);
        vm.expectEmit(true, false, false, true, address(hoprWrapper));
        emit Wrapped(holder, amount);
        xHoprToken.transferAndCall(address(hoprWrapper), amount, hex"00");
        // balance of user address is correct
        assertEq(xHoprToken.balanceOf(holder), INTIAL_BALANCE - amount);
        // total supply equals to the wrapped amount
        assertEq(wxHoprToken.totalSupply(), amount);
    }

    /**
     * @dev Unwrap wxHOPR tokens to xHOPR tokens
     */
    function test_Unwrap(uint256 amount) public {
        amount = bound(amount, 1, 70);
        // wrap 70 xHOPR to wxHOPR
        vm.startPrank(holder);
        xHoprToken.transferAndCall(address(hoprWrapper), 70, hex"00");
        // unwrap some wxHOPR tokens to xHOPR
        vm.expectEmit(true, false, false, true, address(hoprWrapper));
        emit Unwrapped(holder, amount);
        wxHoprToken.transfer(address(hoprWrapper), amount);

        // balance of user address is correct
        assertEq(xHoprToken.balanceOf(address(hoprWrapper)), 70 - amount);
        assertEq(xHoprToken.balanceOf(holder), INTIAL_BALANCE - 70 + amount);
        // total supply equals to the wrapped amount
        assertEq(wxHoprToken.balanceOf(holder), 70 - amount);
        assertEq(wxHoprToken.totalSupply(), 70 - amount);
        // remaining amount of xHOPR in the wrapper
        assertEq(hoprWrapper.xHoprAmount(), 70 - amount);
    }

    /**
     * @dev it should also wrap 5 xHOPR when using "transfer"
     * @notice after the update of Permittable token, it's possible to wrap tokens with "transfer"
     */
    function test_CanAlsoWrapWithTransfer(uint256 amount) public {
        amount = bound(amount, 1, INTIAL_BALANCE);
        vm.prank(holder);
        vm.expectEmit(true, false, false, true, address(hoprWrapper));
        emit Wrapped(holder, amount);
        xHoprToken.transfer(address(hoprWrapper), amount);
        // tokens are transferred
        assertEq(xHoprToken.balanceOf(holder), INTIAL_BALANCE - amount);
        assertEq(xHoprToken.balanceOf(address(hoprWrapper)), amount);
        // but total supply of wxHOPR is not changed
        assertEq(wxHoprToken.totalSupply(), amount);
    }

    /**
     * @dev it should
     * @notice If any token was sent to the wrapper with "transfer" function before the update of PermittableToken,
     * it is possible to recover them with `recoverTokens` function
     */
    function test_RecoverTokens(uint256 amount) public {
        amount = bound(amount, 1, INTIAL_BALANCE - 1);
        vm.prank(holder);
        xHoprToken.transfer(address(hoprWrapper), amount);
        assertEq(hoprWrapper.xHoprAmount(), amount);
        // mock the xHOPR balance of hoprWrapper to INTIAL_BALANCE
        uint256 balanceSlot =
            stdstore.target(address(xHoprToken)).sig("balanceOf(address)").with_key(address(hoprWrapper)).find();
        vm.store(address(xHoprToken), bytes32(balanceSlot), bytes32(abi.encode(INTIAL_BALANCE)));
        // owner can call recoverTokens
        vm.prank(newOwner);
        vm.expectCall(
            address(xHoprToken), abi.encodeWithSignature("transfer(address,uint256)", newOwner, INTIAL_BALANCE - amount)
        );
        hoprWrapper.recoverTokens();
    }

    /**
     * @dev it should fail when sending an unknown "xHOPR" token
     */
    function testRevert_WhenSendingUnknownPermittableToken(address otherPermittableToken, uint256 amount) public {
        amount = bound(amount, 1, INTIAL_BALANCE);
        vm.assume(otherPermittableToken != address(xHoprToken));

        // fail to send tokens to the wrapper
        vm.prank(address(otherPermittableToken));
        vm.expectRevert("Sender must be xHOPR");
        hoprWrapper.onTokenTransfer(holder, amount, hex"00");
    }

    /**
     * @dev it should fail when sending an unknown "wxHOPR" token
     */
    function testRevert_WhenSendingUnknownERC777Token(address otherErc777Token, uint256 amount) public {
        amount = bound(amount, 1, INTIAL_BALANCE);
        vm.assume(otherErc777Token != address(wxHoprToken));

        // fail to send tokens to the wrapper
        vm.prank(address(otherErc777Token));
        vm.expectRevert("Sender must be wxHOPR");
        hoprWrapper.tokensReceived(holder, holder, address(hoprWrapper), amount, hex"00", hex"00");
    }
}
