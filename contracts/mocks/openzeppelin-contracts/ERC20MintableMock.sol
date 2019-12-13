pragma solidity ^0.5.0;

import "../../ERC20Token.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20Mintable.sol";
import "./MinterRoleMock.sol";

contract ERC20MintableMock is ERC20Token, ERC20Mintable, MinterRoleMock {
    // solhint-disable-previous-line no-empty-blocks
}
