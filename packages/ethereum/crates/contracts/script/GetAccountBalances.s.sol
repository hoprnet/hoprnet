// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Script.sol";
import "forge-std/Test.sol";
import "./utils/EnvironmentConfig.s.sol";

contract GetAccountBalancesScript is Test, Script, EnvironmentConfig {
    function run(address account) external returns (uint256 nativeBalance, uint256 tokenBalance) {
        // 1. Environment check
        // get envirionment of the script
        getEnvrionment();
        // read records of deployed files
        readCurrentEnvironment();

        // 2. Check balances
        // check native balance of account;
        uint256 nativeBalance = account.balance;
        // check token balance of account;
        (bool successReadTokenBalance, bytes memory returndataTokenBalance) = currentEnvironmentDetail.hoprTokenContractAddress.staticcall(abi.encodeWithSignature("balanceOf(address)", account));
        if (!successReadTokenBalance) {
            emit log_string("Cannot read balanceOf token contract");
        }
        uint256 tokenBalance = abi.decode(returndataTokenBalance, (uint256));

        // 3. Print out
        emit log_named_address("account", account);
        emit log_named_decimal_uint("native_balance", nativeBalance, 18);
        emit log_named_decimal_uint("token_balance", tokenBalance, 18);

        // 4. return
        return (nativeBalance, tokenBalance);
    }
}
