// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import "../../../src/static/stake/HoprBoost.sol";
import "forge-std/Test.sol";

contract HoprBoostTest is Test {
    // to alter the storage
    using stdStorage for StdStorage;

    HoprBoost public hoprBoost;
    address public admin;
    address public newMinter;
    address[] public accounts = new address[](5);

    // a set of nft types and ranks
    string[2] public types = ["hodlr", "testnet"];
    string[2] public ranks = ["gold", "silver"];
    uint256[3] public numerators = [317, 158, 200];
    uint256 public DEFAULT_DDL = 1_627_387_200;
    string public newBaseURI = "hoprboost.eth.limo/";

    /**
     * Manually import the errors and events
     */
    event Transfer(address indexed _from, address indexed _to, uint256 indexed _tokenId);
    event BoostMinted(uint256 indexed boostTypeIndex, uint256 indexed boostNumerator, uint256 indexed redeemDeadline);

    function setUp() public virtual {
        admin = vm.addr(100); // make address(100) admin
        newMinter = vm.addr(101); // make address(101) a minter

        hoprBoost = new HoprBoost(admin, '');

        // assign vm.addr(1) to vm.addr(5) to accounts
        accounts[0] = vm.addr(1);
        accounts[1] = vm.addr(2);
        accounts[2] = vm.addr(3);
        accounts[3] = vm.addr(4);
        accounts[4] = vm.addr(5);
    }

    /**
     * @dev the contract is created with many correct parameters
     */
    function test_RightParameters() public {
        assertEq(hoprBoost.name(), "HOPR Boost NFT");
        assertEq(hoprBoost.symbol(), "HOPR Boost");
        assertEq(hoprBoost.totalSupply(), 0);

        (uint256 numerator, uint256 ddl) = hoprBoost.boostOf(0);
        assertEq(numerator, 0);
        assertEq(ddl, 0);

        // unminted tokens do not have type index
        assertEq(hoprBoost.typeIndexOf(0), 0);
        assertEq(hoprBoost.typeIndexOf(1), 0);
        assertEq(hoprBoost.typeIndexOf(2), 0);
    }

    /**
     * @dev admin can mint tokens
     */
    function test_AdminCanMint() public {
        // admin is a minter
        assertTrue(hoprBoost.hasRole(hoprBoost.MINTER_ROLE(), admin));
        // admin mint one NFT to accounts[0]
        vm.prank(admin);
        vm.expectEmit(true, true, true, false, address(hoprBoost));
        emit Transfer(address(0), accounts[0], 0);
        vm.expectEmit(true, true, true, false, address(hoprBoost));
        emit BoostMinted(1, numerators[0], DEFAULT_DDL);
        hoprBoost.mint(accounts[0], types[0], ranks[0], numerators[0], DEFAULT_DDL);
        // supply has increased by one
        assertEq(hoprBoost.totalSupply(), 1);
        // has expected boost factors
        (uint256 numerator, uint256 ddl) = hoprBoost.boostOf(0);
        assertEq(numerator, numerators[0]);
        assertEq(ddl, DEFAULT_DDL);
        // it has expected type index
        assertEq(hoprBoost.typeIndexOf(0), 1);
        // it has the correct type name by token id
        assertEq(hoprBoost.typeOf(0), types[0]);
        // it can find type by type index
        assertEq(hoprBoost.typeAt(1), types[0]);
    }

    /**
     * @dev only admin can set base URI
     */
    function testFail_WhenNonAdminSetBaseURI(address account) public {
        vm.assume(account != admin);
        vm.prank(account);
        hoprBoost.updateBaseURI(newBaseURI);
    }

    /**
     * @dev admin can set the base URI. Tokens can read the update
     */
    function test_SetBaseURI() public {
        _helperMintTokens();
        assertEq(hoprBoost.tokenURI(0), _helperBuildURI("", types[0], ranks[0]));
        vm.prank(admin);
        hoprBoost.updateBaseURI(newBaseURI);
        assertEq(hoprBoost.tokenURI(0), _helperBuildURI(newBaseURI, types[0], ranks[0]));
    }

    /**
     * @dev minter can mint exiting (or not) types
     */
    function test_MinterCanBatchMint() public {
        _helperMintTokens();

        address[] memory ownerExistingTypesAndRanks = new address[](2);
        ownerExistingTypesAndRanks[0] = accounts[1];
        ownerExistingTypesAndRanks[1] = accounts[3];
        address[] memory ownerExistingTypes = new address[](2);
        ownerExistingTypes[0] = accounts[1];
        ownerExistingTypes[1] = accounts[2];
        address[] memory ownerNonExistingTypes = new address[](2);
        ownerNonExistingTypes[0] = accounts[2];
        ownerNonExistingTypes[1] = accounts[3];

        vm.startPrank(newMinter);
        // mint two more "hodlr - gold"
        vm.expectEmit(true, true, true, false, address(hoprBoost));
        emit BoostMinted(1, numerators[0], DEFAULT_DDL);
        hoprBoost.batchMint(ownerExistingTypesAndRanks, types[0], ranks[0], numerators[0], DEFAULT_DDL);
        // supply has increased by 2
        assertEq(hoprBoost.totalSupply(), 3);

        // mint two more "hodlr - silver"
        vm.expectEmit(true, true, true, false, address(hoprBoost));
        emit BoostMinted(1, numerators[1], DEFAULT_DDL);
        hoprBoost.batchMint(ownerExistingTypes, types[0], ranks[1], numerators[1], DEFAULT_DDL);
        // supply has increased by 2
        assertEq(hoprBoost.totalSupply(), 5);

        // mint two more "testnet - gold"
        vm.expectEmit(true, true, true, false, address(hoprBoost));
        emit BoostMinted(2, numerators[2], DEFAULT_DDL);
        hoprBoost.batchMint(ownerNonExistingTypes, types[1], ranks[0], numerators[2], DEFAULT_DDL);
        // supply has increased by 2
        assertEq(hoprBoost.totalSupply(), 7);

        for (uint256 i = 0; i < hoprBoost.totalSupply() - 2; i++) {
            // it has expected type index
            assertEq(hoprBoost.typeIndexOf(i), 1);
        }
        for (uint256 j = hoprBoost.totalSupply() - 2; j < hoprBoost.totalSupply(); j++) {
            // it has expected type index
            assertEq(hoprBoost.typeIndexOf(j), 2);
        }
        vm.stopPrank();
    }

    /**
     * @dev Mock erc20 contract and deal some to the hoprBoost
     * Admin can reclaim those ERC20 tokens
     */
    function test_ReclaimERC20Tokens() public {
        address erc20Contract = vm.addr(20);
        vm.mockCall(
            erc20Contract, abi.encodeWithSignature("balanceOf(address)", address(hoprBoost)), abi.encode(1 ether)
        );
        vm.mockCall(
            erc20Contract, abi.encodeWithSignature("transfer(address,uint256)", admin, 1 ether), abi.encode(true)
        );
        vm.prank(admin);
        hoprBoost.reclaimErc20Tokens(erc20Contract);
        vm.clearMockedCalls();
    }

    /**
     * @dev Mock erc721 contract and deal some to the hoprBoost
     * Admin can reclaim those erc721 tokens
     */
    function test_ReclaimERC721Tokens() public {
        address erc721Contract = vm.addr(21);
        vm.mockCall(
            erc721Contract,
            abi.encodeWithSignature("transferFrom(address,address,uint256)", erc721Contract, admin, 30),
            abi.encode(true)
        );
        vm.prank(admin);

        hoprBoost.reclaimErc721Tokens(erc721Contract, 30);
        vm.clearMockedCalls();
    }

    /**
     * @dev mint the following tokens
     *
     *    | NFT type    | hodlr | hodlr  | testnet |
     *    |-------------|-------|--------|---------|
     *    | NFT rank    | gold  | silver | gold    |
     *    |-------------|-------|--------|---------|
     *    | accounts[0] | x     |        |         |
     *    | accounts[1] | x     | x      |         |
     *    | accounts[2] |       | x      | x       |
     *    | accounts[3] | x     |        | x       |
     *    | accounts[4] |       |        |         |
     */
    function _helperMintTokens() internal {
        vm.startPrank(admin);
        hoprBoost.grantRole(hoprBoost.MINTER_ROLE(), newMinter);
        vm.stopPrank();
        vm.prank(newMinter);
        hoprBoost.mint(accounts[0], types[0], ranks[0], numerators[0], DEFAULT_DDL);
    }

    function _helperBuildURI(
        string memory _base,
        string memory _type,
        string memory _rank
    )
        internal
        pure
        returns (string memory)
    {
        return string(abi.encodePacked(_base, _type, "/", _rank));
    }
}
