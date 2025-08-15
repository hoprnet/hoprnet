// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Vm} from "forge-std/Vm.sol";
import {console} from "forge-std/console.sol";

import {StringFinder} from "./StringFinder.sol";

struct ContractInfo {
    /*
     * Contract path, e.g. "src/MyContract.sol"
     */
    string contractPath;
    /*
     * Contract short name, e.g. "MyContract"
     */
    string shortName;
    /*
     * License identifier from the compiled artifact. Empty if not found.
     */
    string license;
    /*
     * keccak256 hash of the source code from metadata
     */
    string sourceCodeHash;
    /*
     * Artifact file path e.g. the path of the file 'out/MyContract.sol/MyContract.json'
     */
    string artifactPath;
}

/**
 * @dev Internal helper methods used by Upgrades and Defender libraries.
 */
library Utils {
    address constant CHEATCODE_ADDRESS = 0x7109709ECfa91a80626fF3989D68f67F5b1DD12D;

    /**
     * @dev Gets the fully qualified name of a contract.
     *
     * @param contractName Contract name in the format "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param outDir Foundry output directory to search in if contractName is not an artifact path
     * @return Fully qualified name of the contract, e.g. "src/MyContract.sol:MyContract"
     */
    function getFullyQualifiedName(
        string memory contractName,
        string memory outDir
    ) internal view returns (string memory) {
        ContractInfo memory info = getContractInfo(contractName, outDir);
        return string(abi.encodePacked(info.contractPath, ":", info.shortName));
    }

    /**
     * @dev Gets information about a contract from its Foundry artifact.
     *
     * @param contractName Contract name in the format "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param outDir Foundry output directory to search in if contractName is not an artifact path
     * @return ContractInfo struct containing information about the contract
     */
    function getContractInfo(
        string memory contractName,
        string memory outDir
    ) internal view returns (ContractInfo memory) {
        Vm vm = Vm(CHEATCODE_ADDRESS);

        ContractInfo memory info;

        info.shortName = _toShortName(contractName);

        string memory fileName = _toFileName(contractName);

        string memory artifactPath = string(
            abi.encodePacked(vm.projectRoot(), "/", outDir, "/", fileName, "/", info.shortName, ".json")
        );
        string memory artifactJson = vm.readFile(artifactPath);

        if (!vm.keyExistsJson(artifactJson, ".ast")) {
            revert(
                string(
                    abi.encodePacked(
                        "Could not find AST in artifact ",
                        artifactPath,
                        ". Set `ast = true` in foundry.toml"
                    )
                )
            );
        }
        info.contractPath = vm.parseJsonString(artifactJson, ".ast.absolutePath");
        if (vm.keyExistsJson(artifactJson, ".ast.license")) {
            info.license = vm.parseJsonString(artifactJson, ".ast.license");
        }
        info.sourceCodeHash = vm.parseJsonString(
            artifactJson,
            string(abi.encodePacked(".metadata.sources.['", info.contractPath, "'].keccak256"))
        );
        info.artifactPath = artifactPath;

        return info;
    }

    using StringFinder for string;

    /**
     * Gets the path to the build-info file that contains the given bytecode.
     *
     * @param sourceCodeHash keccak256 hash of the source code from metadata
     * @param contractName Contract name to display in error message if build-info file is not found
     * @param outDir Foundry output directory that contains a build-info directory
     * @return The path to the build-info file that contains the given bytecode
     */
    function getBuildInfoFile(
        string memory sourceCodeHash,
        string memory contractName,
        string memory outDir
    ) internal returns (string memory) {
        string[] memory inputs = new string[](4);
        inputs[0] = "grep";
        inputs[1] = "-rl";
        inputs[2] = string(abi.encodePacked('"', sourceCodeHash, '"'));
        inputs[3] = string(abi.encodePacked(outDir, "/build-info"));

        Vm.FfiResult memory result = runAsBashCommand(inputs);
        string memory stdout = string(result.stdout);

        if (!stdout.endsWith(".json")) {
            revert(
                string(
                    abi.encodePacked(
                        "Could not find build-info file with matching source code hash for contract ",
                        contractName
                    )
                )
            );
        }

        return stdout;
    }

    /**
     * @dev Gets the output directory from the FOUNDRY_OUT environment variable, or defaults to "out" if not set.
     */
    function getOutDir() internal view returns (string memory) {
        Vm vm = Vm(CHEATCODE_ADDRESS);

        string memory defaultOutDir = "out";
        return vm.envOr("FOUNDRY_OUT", defaultOutDir);
    }

    function _toFileName(string memory name) private pure returns (string memory) {
        Vm vm = Vm(CHEATCODE_ADDRESS);
        if (name.endsWith(".sol")) {
            return name;
        } else if (name.count(":") == 1) {
            return vm.split(name, ":")[0];
        } else {
            if (name.endsWith(".json")) {
                string[] memory parts = vm.split(name, "/");
                if (parts.length > 1) {
                    return parts[parts.length - 2];
                }
            }

            revert(
                string(
                    abi.encodePacked(
                        "Contract name ",
                        name,
                        " must be in the format MyContract.sol:MyContract or MyContract.sol or out/MyContract.sol/MyContract.json"
                    )
                )
            );
        }
    }

    function _toShortName(string memory name) private pure returns (string memory) {
        Vm vm = Vm(CHEATCODE_ADDRESS);
        if (name.endsWith(".sol") && name.count(".sol") == 1) {
            return vm.replace(name, ".sol", "");
        } else if (name.count(":") == 1) {
            return vm.split(name, ":")[1];
        } else if (name.endsWith(".json") && name.count(".json") == 1) {
            string[] memory parts = vm.split(name, "/");
            string memory jsonName = parts[parts.length - 1];
            return vm.replace(jsonName, ".json", "");
        } else {
            revert(
                string(
                    abi.encodePacked(
                        "Contract name ",
                        name,
                        " must be in the format MyContract.sol:MyContract or MyContract.sol or out/MyContract.sol/MyContract.json"
                    )
                )
            );
        }
    }

    /**
     * @dev Converts an array of inputs to a bash command.
     * @param inputs Inputs for a command, e.g. ["grep", "-rl", "0x1234", "out/build-info"]
     * @param bashPath Path to the bash executable or just "bash" if it is in the PATH
     * @return A bash command that runs the given inputs, e.g. ["bash", "-c", "grep -rl 0x1234 out/build-info"]
     */
    function toBashCommand(string[] memory inputs, string memory bashPath) internal pure returns (string[] memory) {
        string memory commandString;
        for (uint i = 0; i < inputs.length; i++) {
            commandString = string(abi.encodePacked(commandString, inputs[i]));
            if (i != inputs.length - 1) {
                commandString = string(abi.encodePacked(commandString, " "));
            }
        }

        string[] memory result = new string[](3);
        result[0] = bashPath;
        result[1] = "-c";
        result[2] = commandString;
        return result;
    }

    /**
     * @dev Runs an arbitrary command using bash.
     * @param inputs Inputs for a command, e.g. ["grep", "-rl", "0x1234", "out/build-info"]
     * @return The result of the corresponding bash command as a Vm.FfiResult struct
     */
    function runAsBashCommand(string[] memory inputs) internal returns (Vm.FfiResult memory) {
        Vm vm = Vm(CHEATCODE_ADDRESS);
        string memory defaultBashPath = "bash";
        string memory bashPath = vm.envOr("OPENZEPPELIN_BASH_PATH", defaultBashPath);

        string[] memory bashCommand = toBashCommand(inputs, bashPath);
        Vm.FfiResult memory result = vm.tryFfi(bashCommand);
        if (result.exitCode != 0 && result.stdout.length == 0 && result.stderr.length == 0) {
            // On Windows, using the bash executable from WSL leads to a non-zero exit code and no output
            revert(
                string(
                    abi.encodePacked(
                        'Failed to run bash command with "',
                        bashCommand[0],
                        '". If you are using Windows, set the OPENZEPPELIN_BASH_PATH environment variable to the fully qualified path of the bash executable. For example, if you are using Git for Windows, add the following line in the .env file of your project (using forward slashes):\nOPENZEPPELIN_BASH_PATH="C:/Program Files/Git/bin/bash"'
                    )
                )
            );
        } else {
            return result;
        }
    }
}
