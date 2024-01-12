// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Script.sol";
import "forge-std/Test.sol";
import "./utils/NetworkConfig.s.sol";

contract GetAccountBalancesScript is Script, Test, NetworkConfig {
    function run(address account) external returns (address wallet, uint256 nativeBalance, uint256 tokenBalance) {
        // 1. Network check
        // get network of the script
        getNetwork();
        // read records of deployed files
        readCurrentNetwork();

        // 2. Check balances
        // check native balance of account;
        nativeBalance = account.balance;
        // check token balance of account;
        (bool successReadTokenBalance, bytes memory returndataTokenBalance) = currentNetworkDetail
            .addresses
            .tokenContractAddress
            .staticcall(abi.encodeWithSignature("balanceOf(address)", account));
        if (!successReadTokenBalance) {
            revert("Cannot read balanceOf token contract");
        }
        tokenBalance = abi.decode(returndataTokenBalance, (uint256));

        // 3. Print out
        emit log_named_address("account", account);
        emit log_named_decimal_uint("native_balance", nativeBalance, 18);
        emit log_named_decimal_uint("token_balance", tokenBalance, 18);

        // 4. return
        return (account, nativeBalance, tokenBalance);
    }
}
