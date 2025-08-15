// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Vm} from "forge-std/Vm.sol";
import {console} from "forge-std/console.sol";

import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

import {Utils, ContractInfo} from "./Utils.sol";
import {Versions} from "./Versions.sol";
import {Options, DefenderOptions} from "../Options.sol";
import {ProposeUpgradeResponse, ApprovalProcessResponse} from "../Defender.sol";

/**
 * @dev Internal helper methods for Defender deployments.
 *
 * WARNING: DO NOT USE DIRECTLY. Use Defender.sol instead.
 */
library DefenderDeploy {
    function deploy(
        string memory contractName,
        bytes memory constructorData,
        DefenderOptions memory defenderOpts
    ) internal returns (address) {
        string memory outDir = Utils.getOutDir();
        ContractInfo memory contractInfo = Utils.getContractInfo(contractName, outDir);
        string memory buildInfoFile = Utils.getBuildInfoFile(
            contractInfo.sourceCodeHash,
            contractInfo.shortName,
            outDir
        );

        string[] memory inputs = buildDeployCommand(contractInfo, buildInfoFile, constructorData, defenderOpts);

        Vm.FfiResult memory result = Utils.runAsBashCommand(inputs);
        string memory stdout = string(result.stdout);

        if (result.exitCode != 0) {
            revert(string(abi.encodePacked("Failed to deploy contract ", contractName, ": ", string(result.stderr))));
        }

        string memory deployedAddress = _parseLine("Deployed to address: ", stdout, true);
        return Vm(Utils.CHEATCODE_ADDRESS).parseAddress(deployedAddress);
    }

    function buildDeployCommand(
        ContractInfo memory contractInfo,
        string memory buildInfoFile,
        bytes memory constructorData,
        DefenderOptions memory defenderOpts
    ) internal view returns (string[] memory) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);

        if (bytes(defenderOpts.licenseType).length != 0) {
            if (defenderOpts.skipVerifySourceCode) {
                revert("The `licenseType` option cannot be used when the `skipVerifySourceCode` option is `true`");
            } else if (defenderOpts.skipLicenseType) {
                revert("The `licenseType` option cannot be used when the `skipLicenseType` option is `true`");
            }
        }

        string[] memory inputBuilder = new string[](255);

        uint8 i = 0;

        inputBuilder[i++] = "npx";
        inputBuilder[i++] = string(
            abi.encodePacked("@openzeppelin/defender-deploy-client-cli@", Versions.DEFENDER_DEPLOY_CLIENT_CLI)
        );
        inputBuilder[i++] = "deploy";
        inputBuilder[i++] = "--contractName";
        inputBuilder[i++] = contractInfo.shortName;
        inputBuilder[i++] = "--contractPath";
        inputBuilder[i++] = contractInfo.contractPath;
        inputBuilder[i++] = "--chainId";
        inputBuilder[i++] = Strings.toString(block.chainid);
        inputBuilder[i++] = "--buildInfoFile";
        inputBuilder[i++] = buildInfoFile;
        if (constructorData.length > 0) {
            inputBuilder[i++] = "--constructorBytecode";
            inputBuilder[i++] = vm.toString(constructorData);
        }
        if (defenderOpts.skipVerifySourceCode) {
            inputBuilder[i++] = "--verifySourceCode";
            inputBuilder[i++] = "false";
        } else if (bytes(defenderOpts.licenseType).length != 0) {
            inputBuilder[i++] = "--licenseType";
            inputBuilder[i++] = string(abi.encodePacked('"', defenderOpts.licenseType, '"'));
        } else if (!defenderOpts.skipLicenseType && bytes(contractInfo.license).length != 0) {
            inputBuilder[i++] = "--licenseType";
            inputBuilder[i++] = string(abi.encodePacked('"', _toLicenseType(contractInfo), '"'));
        }
        if (bytes(defenderOpts.relayerId).length != 0) {
            inputBuilder[i++] = "--relayerId";
            inputBuilder[i++] = defenderOpts.relayerId;
        }
        if (defenderOpts.salt != 0) {
            inputBuilder[i++] = "--salt";
            inputBuilder[i++] = vm.toString(defenderOpts.salt);
        }
        if (defenderOpts.txOverrides.gasLimit != 0) {
            inputBuilder[i++] = "--gasLimit";
            inputBuilder[i++] = Strings.toString(defenderOpts.txOverrides.gasLimit);
        }
        if (defenderOpts.txOverrides.gasPrice != 0) {
            inputBuilder[i++] = "--gasPrice";
            inputBuilder[i++] = Strings.toString(defenderOpts.txOverrides.gasPrice);
        }
        if (defenderOpts.txOverrides.maxFeePerGas != 0) {
            inputBuilder[i++] = "--maxFeePerGas";
            inputBuilder[i++] = Strings.toString(defenderOpts.txOverrides.maxFeePerGas);
        }
        if (defenderOpts.txOverrides.maxPriorityFeePerGas != 0) {
            inputBuilder[i++] = "--maxPriorityFeePerGas";
            inputBuilder[i++] = Strings.toString(defenderOpts.txOverrides.maxPriorityFeePerGas);
        }
        if (bytes(defenderOpts.metadata).length != 0) {
            inputBuilder[i++] = "--metadata";
            inputBuilder[i++] = string(abi.encodePacked('"', vm.replace(defenderOpts.metadata, '"', '\\"'), '"'));
        }
        inputBuilder[i++] = "--origin";
        inputBuilder[i++] = "Foundry";

        // Create a copy of inputs but with the correct length
        string[] memory inputs = new string[](i);
        for (uint8 j = 0; j < i; j++) {
            inputs[j] = inputBuilder[j];
        }

        return inputs;
    }

    using Strings for string;

    function _toLicenseType(ContractInfo memory contractInfo) private pure returns (string memory) {
        string memory id = contractInfo.license;
        if (id.equal("UNLICENSED")) {
            return "None";
        } else if (id.equal("Unlicense")) {
            return "Unlicense";
        } else if (id.equal("MIT")) {
            return "MIT";
        } else if (id.equal("GPL-2.0-only") || id.equal("GPL-2.0-or-later")) {
            return "GNU GPLv2";
        } else if (id.equal("GPL-3.0-only") || id.equal("GPL-3.0-or-later")) {
            return "GNU GPLv3";
        } else if (id.equal("LGPL-2.1-only") || id.equal("LGPL-2.1-or-later")) {
            return "GNU LGPLv2.1";
        } else if (id.equal("LGPL-3.0-only") || id.equal("LGPL-3.0-or-later")) {
            return "GNU LGPLv3";
        } else if (id.equal("BSD-2-Clause")) {
            return "BSD-2-Clause";
        } else if (id.equal("BSD-3-Clause")) {
            return "BSD-3-Clause";
        } else if (id.equal("MPL-2.0")) {
            return "MPL-2.0";
        } else if (id.equal("OSL-3.0")) {
            return "OSL-3.0";
        } else if (id.equal("Apache-2.0")) {
            return "Apache-2.0";
        } else if (id.equal("AGPL-3.0-only") || id.equal("AGPL-3.0-or-later")) {
            return "GNU AGPLv3";
        } else if (id.equal("BUSL-1.1")) {
            return "BSL 1.1";
        } else {
            revert(
                string(
                    abi.encodePacked(
                        "SPDX license identifier ",
                        contractInfo.license,
                        " in ",
                        contractInfo.contractPath,
                        " does not look like a supported license for block explorer verification. Use the `licenseType` option to specify a license type, or set the `skipLicenseType` option to `true` to skip."
                    )
                )
            );
        }
    }

    function proposeUpgrade(
        address proxyAddress,
        address proxyAdminAddress,
        address newImplementationAddress,
        string memory newImplementationContractName,
        Options memory opts
    ) internal returns (ProposeUpgradeResponse memory) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);

        string memory outDir = Utils.getOutDir();
        ContractInfo memory contractInfo = Utils.getContractInfo(newImplementationContractName, outDir);

        string[] memory inputs = buildProposeUpgradeCommand(
            proxyAddress,
            proxyAdminAddress,
            newImplementationAddress,
            contractInfo,
            opts
        );

        Vm.FfiResult memory result = Utils.runAsBashCommand(inputs);
        string memory stdout = string(result.stdout);

        if (result.exitCode != 0) {
            revert(
                string(
                    abi.encodePacked(
                        "Failed to propose upgrade for proxy ",
                        vm.toString(proxyAddress),
                        ": ",
                        string(result.stderr)
                    )
                )
            );
        }

        return parseProposeUpgradeResponse(stdout);
    }

    function parseProposeUpgradeResponse(string memory stdout) internal returns (ProposeUpgradeResponse memory) {
        ProposeUpgradeResponse memory response;
        response.proposalId = _parseLine("Proposal ID: ", stdout, true);
        response.url = _parseLine("Proposal URL: ", stdout, false);
        return response;
    }

    function _parseLine(
        string memory expectedPrefix,
        string memory stdout,
        bool required
    ) private returns (string memory) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);
        if (vm.contains(stdout, expectedPrefix)) {
            // Get the substring after the prefix
            string[] memory segments = vm.split(stdout, expectedPrefix);
            if (segments.length > 2) {
                revert(
                    string(
                        abi.encodePacked(
                            "Found multiple occurrences of prefix '",
                            expectedPrefix,
                            "' in output: ",
                            stdout
                        )
                    )
                );
            }
            string memory suffix = segments[1];
            // Keep only the first line
            return vm.split(suffix, "\n")[0];
        } else if (required) {
            revert(
                string(abi.encodePacked("Failed to find line with prefix '", expectedPrefix, "' in output: ", stdout))
            );
        } else {
            return "";
        }
    }

    function buildProposeUpgradeCommand(
        address proxyAddress,
        address proxyAdminAddress,
        address newImplementationAddress,
        ContractInfo memory contractInfo,
        Options memory opts
    ) internal view returns (string[] memory) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);

        string[] memory inputBuilder = new string[](255);

        uint8 i = 0;

        inputBuilder[i++] = "npx";
        inputBuilder[i++] = string(
            abi.encodePacked("@openzeppelin/defender-deploy-client-cli@", Versions.DEFENDER_DEPLOY_CLIENT_CLI)
        );
        inputBuilder[i++] = "proposeUpgrade";
        inputBuilder[i++] = "--proxyAddress";
        inputBuilder[i++] = vm.toString(proxyAddress);
        inputBuilder[i++] = "--newImplementationAddress";
        inputBuilder[i++] = vm.toString(newImplementationAddress);
        inputBuilder[i++] = "--chainId";
        inputBuilder[i++] = Strings.toString(block.chainid);
        inputBuilder[i++] = "--contractArtifactFile";
        inputBuilder[i++] = string(abi.encodePacked('"', contractInfo.artifactPath, '"'));
        if (proxyAdminAddress != address(0)) {
            inputBuilder[i++] = "--proxyAdminAddress";
            inputBuilder[i++] = vm.toString(proxyAdminAddress);
        }
        if (bytes(opts.defender.upgradeApprovalProcessId).length != 0) {
            inputBuilder[i++] = "--approvalProcessId";
            inputBuilder[i++] = opts.defender.upgradeApprovalProcessId;
        }

        // Create a copy of inputs but with the correct length
        string[] memory inputs = new string[](i);
        for (uint8 j = 0; j < i; j++) {
            inputs[j] = inputBuilder[j];
        }

        return inputs;
    }

    function getApprovalProcess(string memory command) internal returns (ApprovalProcessResponse memory) {
        string[] memory inputs = buildGetApprovalProcessCommand(command);

        Vm.FfiResult memory result = Utils.runAsBashCommand(inputs);
        string memory stdout = string(result.stdout);

        if (result.exitCode != 0) {
            revert(string(abi.encodePacked("Failed to get approval process: ", string(result.stderr))));
        }

        return parseApprovalProcessResponse(stdout);
    }

    function parseApprovalProcessResponse(string memory stdout) internal returns (ApprovalProcessResponse memory) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);

        ApprovalProcessResponse memory response;

        response.approvalProcessId = _parseLine("Approval process ID: ", stdout, true);

        string memory viaString = _parseLine("Via: ", stdout, false);
        if (bytes(viaString).length != 0) {
            response.via = vm.parseAddress(viaString);
        }

        response.viaType = _parseLine("Via type: ", stdout, false);

        return response;
    }

    function buildGetApprovalProcessCommand(string memory command) internal view returns (string[] memory) {
        string[] memory inputBuilder = new string[](255);

        uint8 i = 0;

        inputBuilder[i++] = "npx";
        inputBuilder[i++] = string(
            abi.encodePacked("@openzeppelin/defender-deploy-client-cli@", Versions.DEFENDER_DEPLOY_CLIENT_CLI)
        );
        inputBuilder[i++] = command;
        inputBuilder[i++] = "--chainId";
        inputBuilder[i++] = Strings.toString(block.chainid);

        // Create a copy of inputs but with the correct length
        string[] memory inputs = new string[](i);
        for (uint8 j = 0; j < i; j++) {
            inputs[j] = inputBuilder[j];
        }

        return inputs;
    }
}
