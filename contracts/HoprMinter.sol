pragma solidity ^0.5.3;

import "@openzeppelin/contracts/ownership/Ownable.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "./HoprToken.sol";

contract HoprMinter is Ownable {
    using SafeMath for uint256;

    struct Account {
        uint256 balance;
        uint256 lastClaim;
    }

    // our HoprToken
    HoprToken public token;
    // total amount that is going to be minted -
    // keeps track in order to not reach maxAmount -
    // cannot rely on token.totalSupply due to token burn
    uint256 public amountToMint = 0;
    // max total amount allowed to be minted
    uint256 public maxAmount;
    // when countdown started
    uint256 public started;
    // duration: started + duration = deadline
    uint256 public duration;
    // when an account can claim 100% of it's tokens
    uint256 public deadline;
    // accounts
    mapping(address => Account) public accounts;

    constructor (address _token, uint256 _maxAmount, uint256 _duration) public {
        token = HoprToken(_token);
        maxAmount = _maxAmount;
        started = now;
        duration = _duration;
        deadline = now.add(_duration);
    }

    // Q: should we just move this to a single HoprToken function?
    function replaceMinter(address account) external onlyOwner {
        // add 'account' as minter
        token.addMinter(account);
        // remove this contract from being a minter
        token.renounceMinter();
    }

    // increase balance of an account, this allows the account -
    // to mint specified amount in a linear fashion
    // Q: should we implement 'decreaseBalance'
    function increaseBalance(address account, uint256 amount) external onlyOwner {
        require(now < deadline, "cannot update balances past deadline");

        uint256 newAmountToMint = amountToMint.add(amount);
        require(newAmountToMint <= maxAmount, "reached max amount allowed to be minted");

        amountToMint = newAmountToMint;
        accounts[account].balance = accounts[account].balance.add(amount);
    }

    // 'sender' claims tokens
    function claim() external {
        _claim(msg.sender);
    }

    // 'sender' claims tokens for an account
    function claimFor(address account) external {
        require(account != address(0), "'account' address is empty");

        _claim(account);
    }

    // mints claimable tokens and updates state
    function _claim(address _account) internal {
        Account storage account = accounts[_account];

        uint256 claimable = _claimable(account);
        require(claimable > 0, "nothing to claim");

        require(token.mint(_account, claimable), "minting was unsuccessful");

        account.balance = account.balance.sub(claimable);
        account.lastClaim = now;
    }

    // calculate amount that can be claimed
    function _claimable(Account storage account) internal view returns (uint256) {
        if (now >= deadline) {
            return account.balance;
        }

        bool firstClaim = account.lastClaim == 0;

        if (firstClaim) {
            uint256 since = now.sub(started);

            return since
                .mul(account.balance)
                .div(duration);
        } else {
            uint256 since = now.sub(account.lastClaim);

            return since
                .mul(account.balance)
                .div(deadline.sub(account.lastClaim));
        }
    }
}