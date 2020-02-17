pragma solidity ^0.5.3;

import "@openzeppelin/contracts/token/ERC20/ERC20Mintable.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20Burnable.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20Detailed.sol";
import "./ERC20Token.sol";


contract HoprToken is ERC20Token, ERC20Mintable, ERC20Burnable, ERC20Detailed {
    constructor() public ERC20Detailed("HOPR Token", "HOPR", 18) {}
}
