pub use vm::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod vm {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    pub use super::super::shared_types::*;
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "Vm was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"accesses\",\"outputs\":[{\"internalType\":\"bytes32[]\",\"name\":\"reads\",\"type\":\"bytes32[]\",\"components\":[]},{\"internalType\":\"bytes32[]\",\"name\":\"writes\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"activeFork\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"addr\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"allowCheatcodes\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"assume\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"broadcast\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"broadcast\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"broadcast\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"chainId\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"clearMockedCalls\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"closeFile\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"coinbase\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"createFork\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"createFork\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"createFork\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"createSelectFork\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"createSelectFork\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"createSelectFork\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"deal\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"\",\"type\":\"uint32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"deriveKey\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"\",\"type\":\"uint32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"deriveKey\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"difficulty\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envAddress\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envAddress\",\"outputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envBool\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envBool\",\"outputs\":[{\"internalType\":\"bool[]\",\"name\":\"\",\"type\":\"bool[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envBytes\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envBytes\",\"outputs\":[{\"internalType\":\"bytes[]\",\"name\":\"\",\"type\":\"bytes[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envBytes32\",\"outputs\":[{\"internalType\":\"bytes32[]\",\"name\":\"\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envBytes32\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envInt\",\"outputs\":[{\"internalType\":\"int256[]\",\"name\":\"\",\"type\":\"int256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envInt\",\"outputs\":[{\"internalType\":\"int256\",\"name\":\"\",\"type\":\"int256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envString\",\"outputs\":[{\"internalType\":\"string[]\",\"name\":\"\",\"type\":\"string[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envString\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envUint\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"envUint\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"etch\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"expectCall\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"expectCall\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"expectEmit\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"expectEmit\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"expectRevert\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"expectRevert\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"expectRevert\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"fee\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string[]\",\"name\":\"\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ffi\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"getCode\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"getDeployedCode\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"getNonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"getRecordedLogs\",\"outputs\":[{\"internalType\":\"struct Vm.Log[]\",\"name\":\"\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"bytes32[]\",\"name\":\"topics\",\"type\":\"bytes32[]\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"isPersistent\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"label\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"load\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"makePersistent\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"makePersistent\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"makePersistent\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"makePersistent\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"mockCall\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"mockCall\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"parseAddress\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"parseBool\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"parseBytes\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"parseBytes32\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"parseInt\",\"outputs\":[{\"internalType\":\"int256\",\"name\":\"\",\"type\":\"int256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"parseJson\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"parseJson\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"parseUint\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"prank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"prank\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"projectRoot\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"readFile\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"readFileBinary\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"readLine\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"record\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"recordLogs\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"rememberKey\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"removeFile\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"revertTo\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"revokePersistent\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"revokePersistent\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"roll\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"rollFork\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"forkId\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"rollFork\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"rollFork\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"forkId\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"transaction\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"rollFork\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"rpcUrl\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"rpcUrls\",\"outputs\":[{\"internalType\":\"string[2][]\",\"name\":\"\",\"type\":\"string[2][]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"selectFork\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setEnv\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setNonce\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"sign\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"snapshot\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"startBroadcast\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"startBroadcast\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"startBroadcast\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"startPrank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"startPrank\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"stopBroadcast\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"stopPrank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"store\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"toString\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"toString\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"toString\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"toString\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"int256\",\"name\":\"\",\"type\":\"int256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"toString\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"toString\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"forkId\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"txHash\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transact\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"txHash\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transact\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"warp\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"writeFile\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"writeFileBinary\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"writeLine\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static VM_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct Vm<M>(ethers::contract::Contract<M>);
    impl<M> Clone for Vm<M> {
        fn clone(&self) -> Self {
            Vm(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for Vm<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for Vm<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(Vm))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> Vm<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), VM_ABI.clone(), client).into()
        }
        #[doc = "Calls the contract's `accesses` (0x65bc9481) function"]
        pub fn accesses(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (::std::vec::Vec<[u8; 32]>, ::std::vec::Vec<[u8; 32]>),
        > {
            self.0
                .method_hash([101, 188, 148, 129], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `activeFork` (0x2f103f22) function"]
        pub fn active_fork(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([47, 16, 63, 34], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `addr` (0xffa18649) function"]
        pub fn addr(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([255, 161, 134, 73], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `allowCheatcodes` (0xea060291) function"]
        pub fn allow_cheatcodes(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([234, 6, 2, 145], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `assume` (0x4c63e562) function"]
        pub fn assume(&self, p0: bool) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([76, 99, 229, 98], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `broadcast` (0xafc98040) function"]
        pub fn broadcast_0(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([175, 201, 128, 64], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `broadcast` (0xe6962cdb) function"]
        pub fn broadcast_1(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([230, 150, 44, 219], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `broadcast` (0xf67a965b) function"]
        pub fn broadcast_2(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([246, 122, 150, 91], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `chainId` (0x4049ddd2) function"]
        pub fn chain_id(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([64, 73, 221, 210], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `clearMockedCalls` (0x3fdf4e15) function"]
        pub fn clear_mocked_calls(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([63, 223, 78, 21], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `closeFile` (0x48c3241f) function"]
        pub fn close_file(&self, p0: String) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([72, 195, 36, 31], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `coinbase` (0xff483c54) function"]
        pub fn coinbase(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([255, 72, 60, 84], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `createFork` (0x31ba3498) function"]
        pub fn create_fork_0(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([49, 186, 52, 152], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `createFork` (0x6ba3ba2b) function"]
        pub fn create_fork_1(
            &self,
            p0: String,
            p1: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([107, 163, 186, 43], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `createFork` (0x7ca29682) function"]
        pub fn create_fork_2(
            &self,
            p0: String,
            p1: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([124, 162, 150, 130], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `createSelectFork` (0x71ee464d) function"]
        pub fn create_select_fork_1(
            &self,
            p0: String,
            p1: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([113, 238, 70, 77], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `createSelectFork` (0x84d52b7a) function"]
        pub fn create_select_fork_2(
            &self,
            p0: String,
            p1: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([132, 213, 43, 122], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `createSelectFork` (0x98680034) function"]
        pub fn create_select_fork_0(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([152, 104, 0, 52], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `deal` (0xc88a5e6d) function"]
        pub fn deal(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([200, 138, 94, 109], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `deriveKey` (0x6229498b) function"]
        pub fn derive_key_0(
            &self,
            p0: String,
            p1: u32,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([98, 41, 73, 139], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `deriveKey` (0x6bcb2c1b) function"]
        pub fn derive_key_1(
            &self,
            p0: String,
            p1: String,
            p2: u32,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([107, 203, 44, 27], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `difficulty` (0x46cc92d9) function"]
        pub fn difficulty(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([70, 204, 146, 217], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envAddress` (0x350d56bf) function"]
        pub fn env_address_0(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([53, 13, 86, 191], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envAddress` (0xad31b9fa) function"]
        pub fn env_address_1(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<ethers::core::types::Address>,
        > {
            self.0
                .method_hash([173, 49, 185, 250], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envBool` (0x7ed1ec7d) function"]
        pub fn env_bool_0(&self, p0: String) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([126, 209, 236, 125], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envBool` (0xaaaddeaf) function"]
        pub fn env_bool_1(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<bool>> {
            self.0
                .method_hash([170, 173, 222, 175], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envBytes` (0x4d7baf06) function"]
        pub fn env_bytes_0(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([77, 123, 175, 6], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envBytes` (0xddc2651b) function"]
        pub fn env_bytes_1(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::Bytes>>
        {
            self.0
                .method_hash([221, 194, 101, 27], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envBytes32` (0x5af231c1) function"]
        pub fn env_bytes_321(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<[u8; 32]>> {
            self.0
                .method_hash([90, 242, 49, 193], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envBytes32` (0x97949042) function"]
        pub fn env_bytes_320(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([151, 148, 144, 66], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envInt` (0x42181150) function"]
        pub fn env_int_1(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<I256>> {
            self.0
                .method_hash([66, 24, 17, 80], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envInt` (0x892a0c61) function"]
        pub fn env_int_0(&self, p0: String) -> ethers::contract::builders::ContractCall<M, I256> {
            self.0
                .method_hash([137, 42, 12, 97], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envString` (0x14b02bc9) function"]
        pub fn env_string_1(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<String>> {
            self.0
                .method_hash([20, 176, 43, 201], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envString` (0xf877cb19) function"]
        pub fn env_string_0(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([248, 119, 203, 25], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envUint` (0xc1978d1f) function"]
        pub fn env_uint_0(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([193, 151, 141, 31], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `envUint` (0xf3dec099) function"]
        pub fn env_uint_1(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<ethers::core::types::U256>>
        {
            self.0
                .method_hash([243, 222, 192, 153], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `etch` (0xb4d6c782) function"]
        pub fn etch(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([180, 214, 199, 130], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `expectCall` (0xbd6af434) function"]
        pub fn expect_call_0(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([189, 106, 244, 52], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `expectCall` (0xf30c7ba3) function"]
        pub fn expect_call_1(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::U256,
            p2: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([243, 12, 123, 163], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `expectEmit` (0x491cc7c2) function"]
        pub fn expect_emit_0(
            &self,
            p0: bool,
            p1: bool,
            p2: bool,
            p3: bool,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([73, 28, 199, 194], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `expectEmit` (0x81bad6f3) function"]
        pub fn expect_emit_1(
            &self,
            p0: bool,
            p1: bool,
            p2: bool,
            p3: bool,
            p4: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([129, 186, 214, 243], (p0, p1, p2, p3, p4))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `expectRevert` (0xc31eb0e0) function"]
        pub fn expect_revert_1(
            &self,
            p0: [u8; 4],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([195, 30, 176, 224], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `expectRevert` (0xf28dceb3) function"]
        pub fn expect_revert_2(
            &self,
            p0: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 141, 206, 179], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `expectRevert` (0xf4844814) function"]
        pub fn expect_revert_0(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([244, 132, 72, 20], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `fee` (0x39b37ab0) function"]
        pub fn fee(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([57, 179, 122, 176], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ffi` (0x89160467) function"]
        pub fn ffi(
            &self,
            p0: ::std::vec::Vec<String>,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([137, 22, 4, 103], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCode` (0x8d1cc925) function"]
        pub fn get_code(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([141, 28, 201, 37], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getDeployedCode` (0x3ebf73b4) function"]
        pub fn get_deployed_code(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([62, 191, 115, 180], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getNonce` (0x2d0335ab) function"]
        pub fn get_nonce(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([45, 3, 53, 171], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getRecordedLogs` (0x191553a4) function"]
        pub fn get_recorded_logs(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Log>> {
            self.0
                .method_hash([25, 21, 83, 164], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `isPersistent` (0xd92d8efd) function"]
        pub fn is_persistent(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([217, 45, 142, 253], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `label` (0xc657c718) function"]
        pub fn label(
            &self,
            p0: ethers::core::types::Address,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([198, 87, 199, 24], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `load` (0x667f9d70) function"]
        pub fn load(
            &self,
            p0: ethers::core::types::Address,
            p1: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([102, 127, 157, 112], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `makePersistent` (0x1d9e269e) function"]
        pub fn make_persistent_0(
            &self,
            p0: ::std::vec::Vec<ethers::core::types::Address>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([29, 158, 38, 158], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `makePersistent` (0x4074e0a8) function"]
        pub fn make_persistent_2(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([64, 116, 224, 168], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `makePersistent` (0x57e22dde) function"]
        pub fn make_persistent_1(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([87, 226, 45, 222], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `makePersistent` (0xefb77a75) function"]
        pub fn make_persistent_3(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Address,
            p2: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([239, 183, 122, 117], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `mockCall` (0x81409b91) function"]
        pub fn mock_call_1(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::U256,
            p2: ethers::core::types::Bytes,
            p3: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([129, 64, 155, 145], (p0, p1, p2, p3))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `mockCall` (0xb96213e4) function"]
        pub fn mock_call_0(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Bytes,
            p2: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([185, 98, 19, 228], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `parseAddress` (0xc6ce059d) function"]
        pub fn parse_address(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([198, 206, 5, 157], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `parseBool` (0x974ef924) function"]
        pub fn parse_bool(&self, p0: String) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([151, 78, 249, 36], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `parseBytes` (0x8f5d232d) function"]
        pub fn parse_bytes(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([143, 93, 35, 45], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `parseBytes32` (0x087e6e81) function"]
        pub fn parse_bytes_32(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([8, 126, 110, 129], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `parseInt` (0x42346c5e) function"]
        pub fn parse_int(&self, p0: String) -> ethers::contract::builders::ContractCall<M, I256> {
            self.0
                .method_hash([66, 52, 108, 94], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `parseJson` (0x6a82600a) function"]
        pub fn parse_json_0(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([106, 130, 96, 10], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `parseJson` (0x85940ef1) function"]
        pub fn parse_json_1(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([133, 148, 14, 241], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `parseUint` (0xfa91454d) function"]
        pub fn parse_uint(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([250, 145, 69, 77], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `prank` (0x47e50cce) function"]
        pub fn prank_1(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([71, 229, 12, 206], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `prank` (0xca669fa7) function"]
        pub fn prank_0(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([202, 102, 159, 167], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `projectRoot` (0xd930a0e6) function"]
        pub fn project_root(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([217, 48, 160, 230], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `readFile` (0x60f9bb11) function"]
        pub fn read_file(&self, p0: String) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([96, 249, 187, 17], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `readFileBinary` (0x16ed7bc4) function"]
        pub fn read_file_binary(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([22, 237, 123, 196], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `readLine` (0x70f55728) function"]
        pub fn read_line(&self, p0: String) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([112, 245, 87, 40], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `record` (0x266cf109) function"]
        pub fn record(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([38, 108, 241, 9], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `recordLogs` (0x41af2f52) function"]
        pub fn record_logs(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([65, 175, 47, 82], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `rememberKey` (0x22100064) function"]
        pub fn remember_key(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([34, 16, 0, 100], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `removeFile` (0xf1afe04d) function"]
        pub fn remove_file(&self, p0: String) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([241, 175, 224, 77], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `revertTo` (0x44d7f0a4) function"]
        pub fn revert_to(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([68, 215, 240, 164], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `revokePersistent` (0x3ce969e6) function"]
        pub fn revoke_persistent_0(
            &self,
            p0: ::std::vec::Vec<ethers::core::types::Address>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([60, 233, 105, 230], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `revokePersistent` (0x997a0222) function"]
        pub fn revoke_persistent_1(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([153, 122, 2, 34], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `roll` (0x1f7b4f30) function"]
        pub fn roll(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([31, 123, 79, 48], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `rollFork` (0x0f29772b) function"]
        pub fn roll_fork_0(&self, p0: [u8; 32]) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([15, 41, 119, 43], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `rollFork` (0xd74c83a4) function"]
        pub fn roll_fork_2(
            &self,
            fork_id: ethers::core::types::U256,
            block_number: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([215, 76, 131, 164], (fork_id, block_number))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `rollFork` (0xd9bbf3a1) function"]
        pub fn roll_fork_1(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([217, 187, 243, 161], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `rollFork` (0xf2830f7b) function"]
        pub fn roll_fork_3(
            &self,
            fork_id: ethers::core::types::U256,
            transaction: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 131, 15, 123], (fork_id, transaction))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `rpcUrl` (0x975a6ce9) function"]
        pub fn rpc_url(&self, p0: String) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([151, 90, 108, 233], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `rpcUrls` (0xa85a8418) function"]
        pub fn rpc_urls(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ::std::vec::Vec<[String; 2usize]>>
        {
            self.0
                .method_hash([168, 90, 132, 24], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `selectFork` (0x9ebf6827) function"]
        pub fn select_fork(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([158, 191, 104, 39], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setEnv` (0x3d5923ee) function"]
        pub fn set_env(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([61, 89, 35, 238], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setNonce` (0xf8e18b57) function"]
        pub fn set_nonce(
            &self,
            p0: ethers::core::types::Address,
            p1: u64,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([248, 225, 139, 87], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `sign` (0xe341eaa4) function"]
        pub fn sign(
            &self,
            p0: ethers::core::types::U256,
            p1: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, (u8, [u8; 32], [u8; 32])> {
            self.0
                .method_hash([227, 65, 234, 164], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `snapshot` (0x9711715a) function"]
        pub fn snapshot(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([151, 17, 113, 90], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `startBroadcast` (0x7fb5297f) function"]
        pub fn start_broadcast_0(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([127, 181, 41, 127], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `startBroadcast` (0x7fec2a8d) function"]
        pub fn start_broadcast_1(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([127, 236, 42, 141], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `startBroadcast` (0xce817d47) function"]
        pub fn start_broadcast_2(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([206, 129, 125, 71], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `startPrank` (0x06447d56) function"]
        pub fn start_prank_0(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([6, 68, 125, 86], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `startPrank` (0x45b56078) function"]
        pub fn start_prank_1(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([69, 181, 96, 120], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `stopBroadcast` (0x76eadd36) function"]
        pub fn stop_broadcast(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([118, 234, 221, 54], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `stopPrank` (0x90c5013b) function"]
        pub fn stop_prank(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([144, 197, 1, 59], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `store` (0x70ca10bb) function"]
        pub fn store(
            &self,
            p0: ethers::core::types::Address,
            p1: [u8; 32],
            p2: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([112, 202, 16, 187], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `toString` (0x56ca623e) function"]
        pub fn to_string_0(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([86, 202, 98, 62], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `toString` (0x6900a3ae) function"]
        pub fn to_string_1(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([105, 0, 163, 174], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `toString` (0x71aad10d) function"]
        pub fn to_string_2(
            &self,
            p0: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([113, 170, 209, 13], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `toString` (0x71dce7da) function"]
        pub fn to_string_3(&self, p0: bool) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([113, 220, 231, 218], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `toString` (0xa322c40e) function"]
        pub fn to_string_4(&self, p0: I256) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([163, 34, 196, 14], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `toString` (0xb11a19e8) function"]
        pub fn to_string_5(
            &self,
            p0: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([177, 26, 25, 232], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transact` (0x4d8abc4b) function"]
        pub fn transact_with_fork_id(
            &self,
            fork_id: ethers::core::types::U256,
            tx_hash: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([77, 138, 188, 75], (fork_id, tx_hash))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transact` (0xbe646da1) function"]
        pub fn transact(
            &self,
            tx_hash: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([190, 100, 109, 161], tx_hash)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `warp` (0xe5d6bf02) function"]
        pub fn warp(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([229, 214, 191, 2], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `writeFile` (0x897e0a97) function"]
        pub fn write_file(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([137, 126, 10, 151], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `writeFileBinary` (0x1f21fc80) function"]
        pub fn write_file_binary(
            &self,
            p0: String,
            p1: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([31, 33, 252, 128], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `writeLine` (0x619d897f) function"]
        pub fn write_line(
            &self,
            p0: String,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([97, 157, 137, 127], (p0, p1))
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for Vm<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `accesses` function with signature `accesses(address)` and selector `[101, 188, 148, 129]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "accesses", abi = "accesses(address)")]
    pub struct AccessesCall(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `activeFork` function with signature `activeFork()` and selector `[47, 16, 63, 34]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "activeFork", abi = "activeFork()")]
    pub struct ActiveForkCall;
    #[doc = "Container type for all input parameters for the `addr` function with signature `addr(uint256)` and selector `[255, 161, 134, 73]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "addr", abi = "addr(uint256)")]
    pub struct AddrCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `allowCheatcodes` function with signature `allowCheatcodes(address)` and selector `[234, 6, 2, 145]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "allowCheatcodes", abi = "allowCheatcodes(address)")]
    pub struct AllowCheatcodesCall(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `assume` function with signature `assume(bool)` and selector `[76, 99, 229, 98]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "assume", abi = "assume(bool)")]
    pub struct AssumeCall(pub bool);
    #[doc = "Container type for all input parameters for the `broadcast` function with signature `broadcast()` and selector `[175, 201, 128, 64]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "broadcast", abi = "broadcast()")]
    pub struct Broadcast0Call;
    #[doc = "Container type for all input parameters for the `broadcast` function with signature `broadcast(address)` and selector `[230, 150, 44, 219]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "broadcast", abi = "broadcast(address)")]
    pub struct Broadcast1Call(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `broadcast` function with signature `broadcast(uint256)` and selector `[246, 122, 150, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "broadcast", abi = "broadcast(uint256)")]
    pub struct Broadcast2Call(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `chainId` function with signature `chainId(uint256)` and selector `[64, 73, 221, 210]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "chainId", abi = "chainId(uint256)")]
    pub struct ChainIdCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `clearMockedCalls` function with signature `clearMockedCalls()` and selector `[63, 223, 78, 21]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "clearMockedCalls", abi = "clearMockedCalls()")]
    pub struct ClearMockedCallsCall;
    #[doc = "Container type for all input parameters for the `closeFile` function with signature `closeFile(string)` and selector `[72, 195, 36, 31]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "closeFile", abi = "closeFile(string)")]
    pub struct CloseFileCall(pub String);
    #[doc = "Container type for all input parameters for the `coinbase` function with signature `coinbase(address)` and selector `[255, 72, 60, 84]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "coinbase", abi = "coinbase(address)")]
    pub struct CoinbaseCall(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `createFork` function with signature `createFork(string)` and selector `[49, 186, 52, 152]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "createFork", abi = "createFork(string)")]
    pub struct CreateFork0Call(pub String);
    #[doc = "Container type for all input parameters for the `createFork` function with signature `createFork(string,uint256)` and selector `[107, 163, 186, 43]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "createFork", abi = "createFork(string,uint256)")]
    pub struct CreateFork1Call(pub String, pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `createFork` function with signature `createFork(string,bytes32)` and selector `[124, 162, 150, 130]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "createFork", abi = "createFork(string,bytes32)")]
    pub struct CreateFork2Call(pub String, pub [u8; 32]);
    #[doc = "Container type for all input parameters for the `createSelectFork` function with signature `createSelectFork(string,uint256)` and selector `[113, 238, 70, 77]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "createSelectFork", abi = "createSelectFork(string,uint256)")]
    pub struct CreateSelectFork1Call(pub String, pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `createSelectFork` function with signature `createSelectFork(string,bytes32)` and selector `[132, 213, 43, 122]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "createSelectFork", abi = "createSelectFork(string,bytes32)")]
    pub struct CreateSelectFork2Call(pub String, pub [u8; 32]);
    #[doc = "Container type for all input parameters for the `createSelectFork` function with signature `createSelectFork(string)` and selector `[152, 104, 0, 52]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "createSelectFork", abi = "createSelectFork(string)")]
    pub struct CreateSelectFork0Call(pub String);
    #[doc = "Container type for all input parameters for the `deal` function with signature `deal(address,uint256)` and selector `[200, 138, 94, 109]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "deal", abi = "deal(address,uint256)")]
    pub struct DealCall(
        pub ethers::core::types::Address,
        pub ethers::core::types::U256,
    );
    #[doc = "Container type for all input parameters for the `deriveKey` function with signature `deriveKey(string,uint32)` and selector `[98, 41, 73, 139]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "deriveKey", abi = "deriveKey(string,uint32)")]
    pub struct DeriveKey0Call(pub String, pub u32);
    #[doc = "Container type for all input parameters for the `deriveKey` function with signature `deriveKey(string,string,uint32)` and selector `[107, 203, 44, 27]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "deriveKey", abi = "deriveKey(string,string,uint32)")]
    pub struct DeriveKey1Call(pub String, pub String, pub u32);
    #[doc = "Container type for all input parameters for the `difficulty` function with signature `difficulty(uint256)` and selector `[70, 204, 146, 217]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "difficulty", abi = "difficulty(uint256)")]
    pub struct DifficultyCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `envAddress` function with signature `envAddress(string)` and selector `[53, 13, 86, 191]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envAddress", abi = "envAddress(string)")]
    pub struct EnvAddress0Call(pub String);
    #[doc = "Container type for all input parameters for the `envAddress` function with signature `envAddress(string,string)` and selector `[173, 49, 185, 250]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envAddress", abi = "envAddress(string,string)")]
    pub struct EnvAddress1Call(pub String, pub String);
    #[doc = "Container type for all input parameters for the `envBool` function with signature `envBool(string)` and selector `[126, 209, 236, 125]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envBool", abi = "envBool(string)")]
    pub struct EnvBool0Call(pub String);
    #[doc = "Container type for all input parameters for the `envBool` function with signature `envBool(string,string)` and selector `[170, 173, 222, 175]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envBool", abi = "envBool(string,string)")]
    pub struct EnvBool1Call(pub String, pub String);
    #[doc = "Container type for all input parameters for the `envBytes` function with signature `envBytes(string)` and selector `[77, 123, 175, 6]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envBytes", abi = "envBytes(string)")]
    pub struct EnvBytes0Call(pub String);
    #[doc = "Container type for all input parameters for the `envBytes` function with signature `envBytes(string,string)` and selector `[221, 194, 101, 27]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envBytes", abi = "envBytes(string,string)")]
    pub struct EnvBytes1Call(pub String, pub String);
    #[doc = "Container type for all input parameters for the `envBytes32` function with signature `envBytes32(string,string)` and selector `[90, 242, 49, 193]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envBytes32", abi = "envBytes32(string,string)")]
    pub struct EnvBytes321Call(pub String, pub String);
    #[doc = "Container type for all input parameters for the `envBytes32` function with signature `envBytes32(string)` and selector `[151, 148, 144, 66]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envBytes32", abi = "envBytes32(string)")]
    pub struct EnvBytes320Call(pub String);
    #[doc = "Container type for all input parameters for the `envInt` function with signature `envInt(string,string)` and selector `[66, 24, 17, 80]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envInt", abi = "envInt(string,string)")]
    pub struct EnvInt1Call(pub String, pub String);
    #[doc = "Container type for all input parameters for the `envInt` function with signature `envInt(string)` and selector `[137, 42, 12, 97]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envInt", abi = "envInt(string)")]
    pub struct EnvInt0Call(pub String);
    #[doc = "Container type for all input parameters for the `envString` function with signature `envString(string,string)` and selector `[20, 176, 43, 201]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envString", abi = "envString(string,string)")]
    pub struct EnvString1Call(pub String, pub String);
    #[doc = "Container type for all input parameters for the `envString` function with signature `envString(string)` and selector `[248, 119, 203, 25]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envString", abi = "envString(string)")]
    pub struct EnvString0Call(pub String);
    #[doc = "Container type for all input parameters for the `envUint` function with signature `envUint(string)` and selector `[193, 151, 141, 31]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envUint", abi = "envUint(string)")]
    pub struct EnvUint0Call(pub String);
    #[doc = "Container type for all input parameters for the `envUint` function with signature `envUint(string,string)` and selector `[243, 222, 192, 153]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "envUint", abi = "envUint(string,string)")]
    pub struct EnvUint1Call(pub String, pub String);
    #[doc = "Container type for all input parameters for the `etch` function with signature `etch(address,bytes)` and selector `[180, 214, 199, 130]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "etch", abi = "etch(address,bytes)")]
    pub struct EtchCall(
        pub ethers::core::types::Address,
        pub ethers::core::types::Bytes,
    );
    #[doc = "Container type for all input parameters for the `expectCall` function with signature `expectCall(address,bytes)` and selector `[189, 106, 244, 52]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "expectCall", abi = "expectCall(address,bytes)")]
    pub struct ExpectCall0Call(
        pub ethers::core::types::Address,
        pub ethers::core::types::Bytes,
    );
    #[doc = "Container type for all input parameters for the `expectCall` function with signature `expectCall(address,uint256,bytes)` and selector `[243, 12, 123, 163]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "expectCall", abi = "expectCall(address,uint256,bytes)")]
    pub struct ExpectCall1Call(
        pub ethers::core::types::Address,
        pub ethers::core::types::U256,
        pub ethers::core::types::Bytes,
    );
    #[doc = "Container type for all input parameters for the `expectEmit` function with signature `expectEmit(bool,bool,bool,bool)` and selector `[73, 28, 199, 194]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "expectEmit", abi = "expectEmit(bool,bool,bool,bool)")]
    pub struct ExpectEmit0Call(pub bool, pub bool, pub bool, pub bool);
    #[doc = "Container type for all input parameters for the `expectEmit` function with signature `expectEmit(bool,bool,bool,bool,address)` and selector `[129, 186, 214, 243]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "expectEmit", abi = "expectEmit(bool,bool,bool,bool,address)")]
    pub struct ExpectEmit1Call(
        pub bool,
        pub bool,
        pub bool,
        pub bool,
        pub ethers::core::types::Address,
    );
    #[doc = "Container type for all input parameters for the `expectRevert` function with signature `expectRevert(bytes4)` and selector `[195, 30, 176, 224]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "expectRevert", abi = "expectRevert(bytes4)")]
    pub struct ExpectRevert1Call(pub [u8; 4]);
    #[doc = "Container type for all input parameters for the `expectRevert` function with signature `expectRevert(bytes)` and selector `[242, 141, 206, 179]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "expectRevert", abi = "expectRevert(bytes)")]
    pub struct ExpectRevert2Call(pub ethers::core::types::Bytes);
    #[doc = "Container type for all input parameters for the `expectRevert` function with signature `expectRevert()` and selector `[244, 132, 72, 20]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "expectRevert", abi = "expectRevert()")]
    pub struct ExpectRevert0Call;
    #[doc = "Container type for all input parameters for the `fee` function with signature `fee(uint256)` and selector `[57, 179, 122, 176]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "fee", abi = "fee(uint256)")]
    pub struct FeeCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `ffi` function with signature `ffi(string[])` and selector `[137, 22, 4, 103]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "ffi", abi = "ffi(string[])")]
    pub struct FfiCall(pub ::std::vec::Vec<String>);
    #[doc = "Container type for all input parameters for the `getCode` function with signature `getCode(string)` and selector `[141, 28, 201, 37]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getCode", abi = "getCode(string)")]
    pub struct GetCodeCall(pub String);
    #[doc = "Container type for all input parameters for the `getDeployedCode` function with signature `getDeployedCode(string)` and selector `[62, 191, 115, 180]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getDeployedCode", abi = "getDeployedCode(string)")]
    pub struct GetDeployedCodeCall(pub String);
    #[doc = "Container type for all input parameters for the `getNonce` function with signature `getNonce(address)` and selector `[45, 3, 53, 171]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getNonce", abi = "getNonce(address)")]
    pub struct GetNonceCall(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `getRecordedLogs` function with signature `getRecordedLogs()` and selector `[25, 21, 83, 164]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getRecordedLogs", abi = "getRecordedLogs()")]
    pub struct GetRecordedLogsCall;
    #[doc = "Container type for all input parameters for the `isPersistent` function with signature `isPersistent(address)` and selector `[217, 45, 142, 253]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "isPersistent", abi = "isPersistent(address)")]
    pub struct IsPersistentCall(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `label` function with signature `label(address,string)` and selector `[198, 87, 199, 24]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "label", abi = "label(address,string)")]
    pub struct LabelCall(pub ethers::core::types::Address, pub String);
    #[doc = "Container type for all input parameters for the `load` function with signature `load(address,bytes32)` and selector `[102, 127, 157, 112]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "load", abi = "load(address,bytes32)")]
    pub struct LoadCall(pub ethers::core::types::Address, pub [u8; 32]);
    #[doc = "Container type for all input parameters for the `makePersistent` function with signature `makePersistent(address[])` and selector `[29, 158, 38, 158]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "makePersistent", abi = "makePersistent(address[])")]
    pub struct MakePersistent0Call(pub ::std::vec::Vec<ethers::core::types::Address>);
    #[doc = "Container type for all input parameters for the `makePersistent` function with signature `makePersistent(address,address)` and selector `[64, 116, 224, 168]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "makePersistent", abi = "makePersistent(address,address)")]
    pub struct MakePersistent2Call(
        pub ethers::core::types::Address,
        pub ethers::core::types::Address,
    );
    #[doc = "Container type for all input parameters for the `makePersistent` function with signature `makePersistent(address)` and selector `[87, 226, 45, 222]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "makePersistent", abi = "makePersistent(address)")]
    pub struct MakePersistent1Call(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `makePersistent` function with signature `makePersistent(address,address,address)` and selector `[239, 183, 122, 117]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "makePersistent",
        abi = "makePersistent(address,address,address)"
    )]
    pub struct MakePersistent3Call(
        pub ethers::core::types::Address,
        pub ethers::core::types::Address,
        pub ethers::core::types::Address,
    );
    #[doc = "Container type for all input parameters for the `mockCall` function with signature `mockCall(address,uint256,bytes,bytes)` and selector `[129, 64, 155, 145]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "mockCall", abi = "mockCall(address,uint256,bytes,bytes)")]
    pub struct MockCall1Call(
        pub ethers::core::types::Address,
        pub ethers::core::types::U256,
        pub ethers::core::types::Bytes,
        pub ethers::core::types::Bytes,
    );
    #[doc = "Container type for all input parameters for the `mockCall` function with signature `mockCall(address,bytes,bytes)` and selector `[185, 98, 19, 228]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "mockCall", abi = "mockCall(address,bytes,bytes)")]
    pub struct MockCall0Call(
        pub ethers::core::types::Address,
        pub ethers::core::types::Bytes,
        pub ethers::core::types::Bytes,
    );
    #[doc = "Container type for all input parameters for the `parseAddress` function with signature `parseAddress(string)` and selector `[198, 206, 5, 157]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "parseAddress", abi = "parseAddress(string)")]
    pub struct ParseAddressCall(pub String);
    #[doc = "Container type for all input parameters for the `parseBool` function with signature `parseBool(string)` and selector `[151, 78, 249, 36]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "parseBool", abi = "parseBool(string)")]
    pub struct ParseBoolCall(pub String);
    #[doc = "Container type for all input parameters for the `parseBytes` function with signature `parseBytes(string)` and selector `[143, 93, 35, 45]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "parseBytes", abi = "parseBytes(string)")]
    pub struct ParseBytesCall(pub String);
    #[doc = "Container type for all input parameters for the `parseBytes32` function with signature `parseBytes32(string)` and selector `[8, 126, 110, 129]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "parseBytes32", abi = "parseBytes32(string)")]
    pub struct ParseBytes32Call(pub String);
    #[doc = "Container type for all input parameters for the `parseInt` function with signature `parseInt(string)` and selector `[66, 52, 108, 94]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "parseInt", abi = "parseInt(string)")]
    pub struct ParseIntCall(pub String);
    #[doc = "Container type for all input parameters for the `parseJson` function with signature `parseJson(string)` and selector `[106, 130, 96, 10]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "parseJson", abi = "parseJson(string)")]
    pub struct ParseJson0Call(pub String);
    #[doc = "Container type for all input parameters for the `parseJson` function with signature `parseJson(string,string)` and selector `[133, 148, 14, 241]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "parseJson", abi = "parseJson(string,string)")]
    pub struct ParseJson1Call(pub String, pub String);
    #[doc = "Container type for all input parameters for the `parseUint` function with signature `parseUint(string)` and selector `[250, 145, 69, 77]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "parseUint", abi = "parseUint(string)")]
    pub struct ParseUintCall(pub String);
    #[doc = "Container type for all input parameters for the `prank` function with signature `prank(address,address)` and selector `[71, 229, 12, 206]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "prank", abi = "prank(address,address)")]
    pub struct Prank1Call(
        pub ethers::core::types::Address,
        pub ethers::core::types::Address,
    );
    #[doc = "Container type for all input parameters for the `prank` function with signature `prank(address)` and selector `[202, 102, 159, 167]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "prank", abi = "prank(address)")]
    pub struct Prank0Call(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `projectRoot` function with signature `projectRoot()` and selector `[217, 48, 160, 230]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "projectRoot", abi = "projectRoot()")]
    pub struct ProjectRootCall;
    #[doc = "Container type for all input parameters for the `readFile` function with signature `readFile(string)` and selector `[96, 249, 187, 17]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "readFile", abi = "readFile(string)")]
    pub struct ReadFileCall(pub String);
    #[doc = "Container type for all input parameters for the `readFileBinary` function with signature `readFileBinary(string)` and selector `[22, 237, 123, 196]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "readFileBinary", abi = "readFileBinary(string)")]
    pub struct ReadFileBinaryCall(pub String);
    #[doc = "Container type for all input parameters for the `readLine` function with signature `readLine(string)` and selector `[112, 245, 87, 40]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "readLine", abi = "readLine(string)")]
    pub struct ReadLineCall(pub String);
    #[doc = "Container type for all input parameters for the `record` function with signature `record()` and selector `[38, 108, 241, 9]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "record", abi = "record()")]
    pub struct RecordCall;
    #[doc = "Container type for all input parameters for the `recordLogs` function with signature `recordLogs()` and selector `[65, 175, 47, 82]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "recordLogs", abi = "recordLogs()")]
    pub struct RecordLogsCall;
    #[doc = "Container type for all input parameters for the `rememberKey` function with signature `rememberKey(uint256)` and selector `[34, 16, 0, 100]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "rememberKey", abi = "rememberKey(uint256)")]
    pub struct RememberKeyCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `removeFile` function with signature `removeFile(string)` and selector `[241, 175, 224, 77]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "removeFile", abi = "removeFile(string)")]
    pub struct RemoveFileCall(pub String);
    #[doc = "Container type for all input parameters for the `revertTo` function with signature `revertTo(uint256)` and selector `[68, 215, 240, 164]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "revertTo", abi = "revertTo(uint256)")]
    pub struct RevertToCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `revokePersistent` function with signature `revokePersistent(address[])` and selector `[60, 233, 105, 230]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "revokePersistent", abi = "revokePersistent(address[])")]
    pub struct RevokePersistent0Call(pub ::std::vec::Vec<ethers::core::types::Address>);
    #[doc = "Container type for all input parameters for the `revokePersistent` function with signature `revokePersistent(address)` and selector `[153, 122, 2, 34]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "revokePersistent", abi = "revokePersistent(address)")]
    pub struct RevokePersistent1Call(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `roll` function with signature `roll(uint256)` and selector `[31, 123, 79, 48]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "roll", abi = "roll(uint256)")]
    pub struct RollCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `rollFork` function with signature `rollFork(bytes32)` and selector `[15, 41, 119, 43]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "rollFork", abi = "rollFork(bytes32)")]
    pub struct RollFork0Call(pub [u8; 32]);
    #[doc = "Container type for all input parameters for the `rollFork` function with signature `rollFork(uint256,uint256)` and selector `[215, 76, 131, 164]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "rollFork", abi = "rollFork(uint256,uint256)")]
    pub struct RollFork2Call {
        pub fork_id: ethers::core::types::U256,
        pub block_number: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `rollFork` function with signature `rollFork(uint256)` and selector `[217, 187, 243, 161]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "rollFork", abi = "rollFork(uint256)")]
    pub struct RollFork1Call(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `rollFork` function with signature `rollFork(uint256,bytes32)` and selector `[242, 131, 15, 123]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "rollFork", abi = "rollFork(uint256,bytes32)")]
    pub struct RollFork3Call {
        pub fork_id: ethers::core::types::U256,
        pub transaction: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `rpcUrl` function with signature `rpcUrl(string)` and selector `[151, 90, 108, 233]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "rpcUrl", abi = "rpcUrl(string)")]
    pub struct RpcUrlCall(pub String);
    #[doc = "Container type for all input parameters for the `rpcUrls` function with signature `rpcUrls()` and selector `[168, 90, 132, 24]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "rpcUrls", abi = "rpcUrls()")]
    pub struct RpcUrlsCall;
    #[doc = "Container type for all input parameters for the `selectFork` function with signature `selectFork(uint256)` and selector `[158, 191, 104, 39]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "selectFork", abi = "selectFork(uint256)")]
    pub struct SelectForkCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `setEnv` function with signature `setEnv(string,string)` and selector `[61, 89, 35, 238]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setEnv", abi = "setEnv(string,string)")]
    pub struct SetEnvCall(pub String, pub String);
    #[doc = "Container type for all input parameters for the `setNonce` function with signature `setNonce(address,uint64)` and selector `[248, 225, 139, 87]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setNonce", abi = "setNonce(address,uint64)")]
    pub struct SetNonceCall(pub ethers::core::types::Address, pub u64);
    #[doc = "Container type for all input parameters for the `sign` function with signature `sign(uint256,bytes32)` and selector `[227, 65, 234, 164]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "sign", abi = "sign(uint256,bytes32)")]
    pub struct SignCall(pub ethers::core::types::U256, pub [u8; 32]);
    #[doc = "Container type for all input parameters for the `snapshot` function with signature `snapshot()` and selector `[151, 17, 113, 90]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "snapshot", abi = "snapshot()")]
    pub struct SnapshotCall;
    #[doc = "Container type for all input parameters for the `startBroadcast` function with signature `startBroadcast()` and selector `[127, 181, 41, 127]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "startBroadcast", abi = "startBroadcast()")]
    pub struct StartBroadcast0Call;
    #[doc = "Container type for all input parameters for the `startBroadcast` function with signature `startBroadcast(address)` and selector `[127, 236, 42, 141]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "startBroadcast", abi = "startBroadcast(address)")]
    pub struct StartBroadcast1Call(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `startBroadcast` function with signature `startBroadcast(uint256)` and selector `[206, 129, 125, 71]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "startBroadcast", abi = "startBroadcast(uint256)")]
    pub struct StartBroadcast2Call(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `startPrank` function with signature `startPrank(address)` and selector `[6, 68, 125, 86]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "startPrank", abi = "startPrank(address)")]
    pub struct StartPrank0Call(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `startPrank` function with signature `startPrank(address,address)` and selector `[69, 181, 96, 120]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "startPrank", abi = "startPrank(address,address)")]
    pub struct StartPrank1Call(
        pub ethers::core::types::Address,
        pub ethers::core::types::Address,
    );
    #[doc = "Container type for all input parameters for the `stopBroadcast` function with signature `stopBroadcast()` and selector `[118, 234, 221, 54]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "stopBroadcast", abi = "stopBroadcast()")]
    pub struct StopBroadcastCall;
    #[doc = "Container type for all input parameters for the `stopPrank` function with signature `stopPrank()` and selector `[144, 197, 1, 59]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "stopPrank", abi = "stopPrank()")]
    pub struct StopPrankCall;
    #[doc = "Container type for all input parameters for the `store` function with signature `store(address,bytes32,bytes32)` and selector `[112, 202, 16, 187]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "store", abi = "store(address,bytes32,bytes32)")]
    pub struct StoreCall(pub ethers::core::types::Address, pub [u8; 32], pub [u8; 32]);
    #[doc = "Container type for all input parameters for the `toString` function with signature `toString(address)` and selector `[86, 202, 98, 62]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "toString", abi = "toString(address)")]
    pub struct ToString0Call(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `toString` function with signature `toString(uint256)` and selector `[105, 0, 163, 174]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "toString", abi = "toString(uint256)")]
    pub struct ToString1Call(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `toString` function with signature `toString(bytes)` and selector `[113, 170, 209, 13]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "toString", abi = "toString(bytes)")]
    pub struct ToString2Call(pub ethers::core::types::Bytes);
    #[doc = "Container type for all input parameters for the `toString` function with signature `toString(bool)` and selector `[113, 220, 231, 218]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "toString", abi = "toString(bool)")]
    pub struct ToString3Call(pub bool);
    #[doc = "Container type for all input parameters for the `toString` function with signature `toString(int256)` and selector `[163, 34, 196, 14]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "toString", abi = "toString(int256)")]
    pub struct ToString4Call(pub I256);
    #[doc = "Container type for all input parameters for the `toString` function with signature `toString(bytes32)` and selector `[177, 26, 25, 232]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "toString", abi = "toString(bytes32)")]
    pub struct ToString5Call(pub [u8; 32]);
    #[doc = "Container type for all input parameters for the `transact` function with signature `transact(uint256,bytes32)` and selector `[77, 138, 188, 75]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transact", abi = "transact(uint256,bytes32)")]
    pub struct TransactWithForkIdCall {
        pub fork_id: ethers::core::types::U256,
        pub tx_hash: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `transact` function with signature `transact(bytes32)` and selector `[190, 100, 109, 161]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transact", abi = "transact(bytes32)")]
    pub struct TransactCall {
        pub tx_hash: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `warp` function with signature `warp(uint256)` and selector `[229, 214, 191, 2]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "warp", abi = "warp(uint256)")]
    pub struct WarpCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `writeFile` function with signature `writeFile(string,string)` and selector `[137, 126, 10, 151]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "writeFile", abi = "writeFile(string,string)")]
    pub struct WriteFileCall(pub String, pub String);
    #[doc = "Container type for all input parameters for the `writeFileBinary` function with signature `writeFileBinary(string,bytes)` and selector `[31, 33, 252, 128]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "writeFileBinary", abi = "writeFileBinary(string,bytes)")]
    pub struct WriteFileBinaryCall(pub String, pub ethers::core::types::Bytes);
    #[doc = "Container type for all input parameters for the `writeLine` function with signature `writeLine(string,string)` and selector `[97, 157, 137, 127]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "writeLine", abi = "writeLine(string,string)")]
    pub struct WriteLineCall(pub String, pub String);
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum VmCalls {
        Accesses(AccessesCall),
        ActiveFork(ActiveForkCall),
        Addr(AddrCall),
        AllowCheatcodes(AllowCheatcodesCall),
        Assume(AssumeCall),
        Broadcast0(Broadcast0Call),
        Broadcast1(Broadcast1Call),
        Broadcast2(Broadcast2Call),
        ChainId(ChainIdCall),
        ClearMockedCalls(ClearMockedCallsCall),
        CloseFile(CloseFileCall),
        Coinbase(CoinbaseCall),
        CreateFork0(CreateFork0Call),
        CreateFork1(CreateFork1Call),
        CreateFork2(CreateFork2Call),
        CreateSelectFork1(CreateSelectFork1Call),
        CreateSelectFork2(CreateSelectFork2Call),
        CreateSelectFork0(CreateSelectFork0Call),
        Deal(DealCall),
        DeriveKey0(DeriveKey0Call),
        DeriveKey1(DeriveKey1Call),
        Difficulty(DifficultyCall),
        EnvAddress0(EnvAddress0Call),
        EnvAddress1(EnvAddress1Call),
        EnvBool0(EnvBool0Call),
        EnvBool1(EnvBool1Call),
        EnvBytes0(EnvBytes0Call),
        EnvBytes1(EnvBytes1Call),
        EnvBytes321(EnvBytes321Call),
        EnvBytes320(EnvBytes320Call),
        EnvInt1(EnvInt1Call),
        EnvInt0(EnvInt0Call),
        EnvString1(EnvString1Call),
        EnvString0(EnvString0Call),
        EnvUint0(EnvUint0Call),
        EnvUint1(EnvUint1Call),
        Etch(EtchCall),
        ExpectCall0(ExpectCall0Call),
        ExpectCall1(ExpectCall1Call),
        ExpectEmit0(ExpectEmit0Call),
        ExpectEmit1(ExpectEmit1Call),
        ExpectRevert1(ExpectRevert1Call),
        ExpectRevert2(ExpectRevert2Call),
        ExpectRevert0(ExpectRevert0Call),
        Fee(FeeCall),
        Ffi(FfiCall),
        GetCode(GetCodeCall),
        GetDeployedCode(GetDeployedCodeCall),
        GetNonce(GetNonceCall),
        GetRecordedLogs(GetRecordedLogsCall),
        IsPersistent(IsPersistentCall),
        Label(LabelCall),
        Load(LoadCall),
        MakePersistent0(MakePersistent0Call),
        MakePersistent2(MakePersistent2Call),
        MakePersistent1(MakePersistent1Call),
        MakePersistent3(MakePersistent3Call),
        MockCall1(MockCall1Call),
        MockCall0(MockCall0Call),
        ParseAddress(ParseAddressCall),
        ParseBool(ParseBoolCall),
        ParseBytes(ParseBytesCall),
        ParseBytes32(ParseBytes32Call),
        ParseInt(ParseIntCall),
        ParseJson0(ParseJson0Call),
        ParseJson1(ParseJson1Call),
        ParseUint(ParseUintCall),
        Prank1(Prank1Call),
        Prank0(Prank0Call),
        ProjectRoot(ProjectRootCall),
        ReadFile(ReadFileCall),
        ReadFileBinary(ReadFileBinaryCall),
        ReadLine(ReadLineCall),
        Record(RecordCall),
        RecordLogs(RecordLogsCall),
        RememberKey(RememberKeyCall),
        RemoveFile(RemoveFileCall),
        RevertTo(RevertToCall),
        RevokePersistent0(RevokePersistent0Call),
        RevokePersistent1(RevokePersistent1Call),
        Roll(RollCall),
        RollFork0(RollFork0Call),
        RollFork2(RollFork2Call),
        RollFork1(RollFork1Call),
        RollFork3(RollFork3Call),
        RpcUrl(RpcUrlCall),
        RpcUrls(RpcUrlsCall),
        SelectFork(SelectForkCall),
        SetEnv(SetEnvCall),
        SetNonce(SetNonceCall),
        Sign(SignCall),
        Snapshot(SnapshotCall),
        StartBroadcast0(StartBroadcast0Call),
        StartBroadcast1(StartBroadcast1Call),
        StartBroadcast2(StartBroadcast2Call),
        StartPrank0(StartPrank0Call),
        StartPrank1(StartPrank1Call),
        StopBroadcast(StopBroadcastCall),
        StopPrank(StopPrankCall),
        Store(StoreCall),
        ToString0(ToString0Call),
        ToString1(ToString1Call),
        ToString2(ToString2Call),
        ToString3(ToString3Call),
        ToString4(ToString4Call),
        ToString5(ToString5Call),
        TransactWithForkId(TransactWithForkIdCall),
        Transact(TransactCall),
        Warp(WarpCall),
        WriteFile(WriteFileCall),
        WriteFileBinary(WriteFileBinaryCall),
        WriteLine(WriteLineCall),
    }
    impl ethers::core::abi::AbiDecode for VmCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <AccessesCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Accesses(decoded));
            }
            if let Ok(decoded) =
                <ActiveForkCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ActiveFork(decoded));
            }
            if let Ok(decoded) = <AddrCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(VmCalls::Addr(decoded));
            }
            if let Ok(decoded) =
                <AllowCheatcodesCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::AllowCheatcodes(decoded));
            }
            if let Ok(decoded) = <AssumeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Assume(decoded));
            }
            if let Ok(decoded) =
                <Broadcast0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Broadcast0(decoded));
            }
            if let Ok(decoded) =
                <Broadcast1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Broadcast1(decoded));
            }
            if let Ok(decoded) =
                <Broadcast2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Broadcast2(decoded));
            }
            if let Ok(decoded) =
                <ChainIdCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ChainId(decoded));
            }
            if let Ok(decoded) =
                <ClearMockedCallsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ClearMockedCalls(decoded));
            }
            if let Ok(decoded) =
                <CloseFileCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::CloseFile(decoded));
            }
            if let Ok(decoded) =
                <CoinbaseCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Coinbase(decoded));
            }
            if let Ok(decoded) =
                <CreateFork0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::CreateFork0(decoded));
            }
            if let Ok(decoded) =
                <CreateFork1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::CreateFork1(decoded));
            }
            if let Ok(decoded) =
                <CreateFork2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::CreateFork2(decoded));
            }
            if let Ok(decoded) =
                <CreateSelectFork1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::CreateSelectFork1(decoded));
            }
            if let Ok(decoded) =
                <CreateSelectFork2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::CreateSelectFork2(decoded));
            }
            if let Ok(decoded) =
                <CreateSelectFork0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::CreateSelectFork0(decoded));
            }
            if let Ok(decoded) = <DealCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(VmCalls::Deal(decoded));
            }
            if let Ok(decoded) =
                <DeriveKey0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::DeriveKey0(decoded));
            }
            if let Ok(decoded) =
                <DeriveKey1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::DeriveKey1(decoded));
            }
            if let Ok(decoded) =
                <DifficultyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Difficulty(decoded));
            }
            if let Ok(decoded) =
                <EnvAddress0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvAddress0(decoded));
            }
            if let Ok(decoded) =
                <EnvAddress1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvAddress1(decoded));
            }
            if let Ok(decoded) =
                <EnvBool0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvBool0(decoded));
            }
            if let Ok(decoded) =
                <EnvBool1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvBool1(decoded));
            }
            if let Ok(decoded) =
                <EnvBytes0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvBytes0(decoded));
            }
            if let Ok(decoded) =
                <EnvBytes1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvBytes1(decoded));
            }
            if let Ok(decoded) =
                <EnvBytes321Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvBytes321(decoded));
            }
            if let Ok(decoded) =
                <EnvBytes320Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvBytes320(decoded));
            }
            if let Ok(decoded) =
                <EnvInt1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvInt1(decoded));
            }
            if let Ok(decoded) =
                <EnvInt0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvInt0(decoded));
            }
            if let Ok(decoded) =
                <EnvString1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvString1(decoded));
            }
            if let Ok(decoded) =
                <EnvString0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvString0(decoded));
            }
            if let Ok(decoded) =
                <EnvUint0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvUint0(decoded));
            }
            if let Ok(decoded) =
                <EnvUint1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::EnvUint1(decoded));
            }
            if let Ok(decoded) = <EtchCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(VmCalls::Etch(decoded));
            }
            if let Ok(decoded) =
                <ExpectCall0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ExpectCall0(decoded));
            }
            if let Ok(decoded) =
                <ExpectCall1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ExpectCall1(decoded));
            }
            if let Ok(decoded) =
                <ExpectEmit0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ExpectEmit0(decoded));
            }
            if let Ok(decoded) =
                <ExpectEmit1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ExpectEmit1(decoded));
            }
            if let Ok(decoded) =
                <ExpectRevert1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ExpectRevert1(decoded));
            }
            if let Ok(decoded) =
                <ExpectRevert2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ExpectRevert2(decoded));
            }
            if let Ok(decoded) =
                <ExpectRevert0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ExpectRevert0(decoded));
            }
            if let Ok(decoded) = <FeeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(VmCalls::Fee(decoded));
            }
            if let Ok(decoded) = <FfiCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(VmCalls::Ffi(decoded));
            }
            if let Ok(decoded) =
                <GetCodeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::GetCode(decoded));
            }
            if let Ok(decoded) =
                <GetDeployedCodeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::GetDeployedCode(decoded));
            }
            if let Ok(decoded) =
                <GetNonceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::GetNonce(decoded));
            }
            if let Ok(decoded) =
                <GetRecordedLogsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::GetRecordedLogs(decoded));
            }
            if let Ok(decoded) =
                <IsPersistentCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::IsPersistent(decoded));
            }
            if let Ok(decoded) = <LabelCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Label(decoded));
            }
            if let Ok(decoded) = <LoadCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(VmCalls::Load(decoded));
            }
            if let Ok(decoded) =
                <MakePersistent0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::MakePersistent0(decoded));
            }
            if let Ok(decoded) =
                <MakePersistent2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::MakePersistent2(decoded));
            }
            if let Ok(decoded) =
                <MakePersistent1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::MakePersistent1(decoded));
            }
            if let Ok(decoded) =
                <MakePersistent3Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::MakePersistent3(decoded));
            }
            if let Ok(decoded) =
                <MockCall1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::MockCall1(decoded));
            }
            if let Ok(decoded) =
                <MockCall0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::MockCall0(decoded));
            }
            if let Ok(decoded) =
                <ParseAddressCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ParseAddress(decoded));
            }
            if let Ok(decoded) =
                <ParseBoolCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ParseBool(decoded));
            }
            if let Ok(decoded) =
                <ParseBytesCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ParseBytes(decoded));
            }
            if let Ok(decoded) =
                <ParseBytes32Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ParseBytes32(decoded));
            }
            if let Ok(decoded) =
                <ParseIntCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ParseInt(decoded));
            }
            if let Ok(decoded) =
                <ParseJson0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ParseJson0(decoded));
            }
            if let Ok(decoded) =
                <ParseJson1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ParseJson1(decoded));
            }
            if let Ok(decoded) =
                <ParseUintCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ParseUint(decoded));
            }
            if let Ok(decoded) = <Prank1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Prank1(decoded));
            }
            if let Ok(decoded) = <Prank0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Prank0(decoded));
            }
            if let Ok(decoded) =
                <ProjectRootCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ProjectRoot(decoded));
            }
            if let Ok(decoded) =
                <ReadFileCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ReadFile(decoded));
            }
            if let Ok(decoded) =
                <ReadFileBinaryCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ReadFileBinary(decoded));
            }
            if let Ok(decoded) =
                <ReadLineCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ReadLine(decoded));
            }
            if let Ok(decoded) = <RecordCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Record(decoded));
            }
            if let Ok(decoded) =
                <RecordLogsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RecordLogs(decoded));
            }
            if let Ok(decoded) =
                <RememberKeyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RememberKey(decoded));
            }
            if let Ok(decoded) =
                <RemoveFileCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RemoveFile(decoded));
            }
            if let Ok(decoded) =
                <RevertToCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RevertTo(decoded));
            }
            if let Ok(decoded) =
                <RevokePersistent0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RevokePersistent0(decoded));
            }
            if let Ok(decoded) =
                <RevokePersistent1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RevokePersistent1(decoded));
            }
            if let Ok(decoded) = <RollCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(VmCalls::Roll(decoded));
            }
            if let Ok(decoded) =
                <RollFork0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RollFork0(decoded));
            }
            if let Ok(decoded) =
                <RollFork2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RollFork2(decoded));
            }
            if let Ok(decoded) =
                <RollFork1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RollFork1(decoded));
            }
            if let Ok(decoded) =
                <RollFork3Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RollFork3(decoded));
            }
            if let Ok(decoded) = <RpcUrlCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RpcUrl(decoded));
            }
            if let Ok(decoded) =
                <RpcUrlsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::RpcUrls(decoded));
            }
            if let Ok(decoded) =
                <SelectForkCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::SelectFork(decoded));
            }
            if let Ok(decoded) = <SetEnvCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::SetEnv(decoded));
            }
            if let Ok(decoded) =
                <SetNonceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::SetNonce(decoded));
            }
            if let Ok(decoded) = <SignCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(VmCalls::Sign(decoded));
            }
            if let Ok(decoded) =
                <SnapshotCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Snapshot(decoded));
            }
            if let Ok(decoded) =
                <StartBroadcast0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::StartBroadcast0(decoded));
            }
            if let Ok(decoded) =
                <StartBroadcast1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::StartBroadcast1(decoded));
            }
            if let Ok(decoded) =
                <StartBroadcast2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::StartBroadcast2(decoded));
            }
            if let Ok(decoded) =
                <StartPrank0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::StartPrank0(decoded));
            }
            if let Ok(decoded) =
                <StartPrank1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::StartPrank1(decoded));
            }
            if let Ok(decoded) =
                <StopBroadcastCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::StopBroadcast(decoded));
            }
            if let Ok(decoded) =
                <StopPrankCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::StopPrank(decoded));
            }
            if let Ok(decoded) = <StoreCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Store(decoded));
            }
            if let Ok(decoded) =
                <ToString0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ToString0(decoded));
            }
            if let Ok(decoded) =
                <ToString1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ToString1(decoded));
            }
            if let Ok(decoded) =
                <ToString2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ToString2(decoded));
            }
            if let Ok(decoded) =
                <ToString3Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ToString3(decoded));
            }
            if let Ok(decoded) =
                <ToString4Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ToString4(decoded));
            }
            if let Ok(decoded) =
                <ToString5Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::ToString5(decoded));
            }
            if let Ok(decoded) =
                <TransactWithForkIdCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::TransactWithForkId(decoded));
            }
            if let Ok(decoded) =
                <TransactCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::Transact(decoded));
            }
            if let Ok(decoded) = <WarpCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(VmCalls::Warp(decoded));
            }
            if let Ok(decoded) =
                <WriteFileCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::WriteFile(decoded));
            }
            if let Ok(decoded) =
                <WriteFileBinaryCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::WriteFileBinary(decoded));
            }
            if let Ok(decoded) =
                <WriteLineCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(VmCalls::WriteLine(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for VmCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                VmCalls::Accesses(element) => element.encode(),
                VmCalls::ActiveFork(element) => element.encode(),
                VmCalls::Addr(element) => element.encode(),
                VmCalls::AllowCheatcodes(element) => element.encode(),
                VmCalls::Assume(element) => element.encode(),
                VmCalls::Broadcast0(element) => element.encode(),
                VmCalls::Broadcast1(element) => element.encode(),
                VmCalls::Broadcast2(element) => element.encode(),
                VmCalls::ChainId(element) => element.encode(),
                VmCalls::ClearMockedCalls(element) => element.encode(),
                VmCalls::CloseFile(element) => element.encode(),
                VmCalls::Coinbase(element) => element.encode(),
                VmCalls::CreateFork0(element) => element.encode(),
                VmCalls::CreateFork1(element) => element.encode(),
                VmCalls::CreateFork2(element) => element.encode(),
                VmCalls::CreateSelectFork1(element) => element.encode(),
                VmCalls::CreateSelectFork2(element) => element.encode(),
                VmCalls::CreateSelectFork0(element) => element.encode(),
                VmCalls::Deal(element) => element.encode(),
                VmCalls::DeriveKey0(element) => element.encode(),
                VmCalls::DeriveKey1(element) => element.encode(),
                VmCalls::Difficulty(element) => element.encode(),
                VmCalls::EnvAddress0(element) => element.encode(),
                VmCalls::EnvAddress1(element) => element.encode(),
                VmCalls::EnvBool0(element) => element.encode(),
                VmCalls::EnvBool1(element) => element.encode(),
                VmCalls::EnvBytes0(element) => element.encode(),
                VmCalls::EnvBytes1(element) => element.encode(),
                VmCalls::EnvBytes321(element) => element.encode(),
                VmCalls::EnvBytes320(element) => element.encode(),
                VmCalls::EnvInt1(element) => element.encode(),
                VmCalls::EnvInt0(element) => element.encode(),
                VmCalls::EnvString1(element) => element.encode(),
                VmCalls::EnvString0(element) => element.encode(),
                VmCalls::EnvUint0(element) => element.encode(),
                VmCalls::EnvUint1(element) => element.encode(),
                VmCalls::Etch(element) => element.encode(),
                VmCalls::ExpectCall0(element) => element.encode(),
                VmCalls::ExpectCall1(element) => element.encode(),
                VmCalls::ExpectEmit0(element) => element.encode(),
                VmCalls::ExpectEmit1(element) => element.encode(),
                VmCalls::ExpectRevert1(element) => element.encode(),
                VmCalls::ExpectRevert2(element) => element.encode(),
                VmCalls::ExpectRevert0(element) => element.encode(),
                VmCalls::Fee(element) => element.encode(),
                VmCalls::Ffi(element) => element.encode(),
                VmCalls::GetCode(element) => element.encode(),
                VmCalls::GetDeployedCode(element) => element.encode(),
                VmCalls::GetNonce(element) => element.encode(),
                VmCalls::GetRecordedLogs(element) => element.encode(),
                VmCalls::IsPersistent(element) => element.encode(),
                VmCalls::Label(element) => element.encode(),
                VmCalls::Load(element) => element.encode(),
                VmCalls::MakePersistent0(element) => element.encode(),
                VmCalls::MakePersistent2(element) => element.encode(),
                VmCalls::MakePersistent1(element) => element.encode(),
                VmCalls::MakePersistent3(element) => element.encode(),
                VmCalls::MockCall1(element) => element.encode(),
                VmCalls::MockCall0(element) => element.encode(),
                VmCalls::ParseAddress(element) => element.encode(),
                VmCalls::ParseBool(element) => element.encode(),
                VmCalls::ParseBytes(element) => element.encode(),
                VmCalls::ParseBytes32(element) => element.encode(),
                VmCalls::ParseInt(element) => element.encode(),
                VmCalls::ParseJson0(element) => element.encode(),
                VmCalls::ParseJson1(element) => element.encode(),
                VmCalls::ParseUint(element) => element.encode(),
                VmCalls::Prank1(element) => element.encode(),
                VmCalls::Prank0(element) => element.encode(),
                VmCalls::ProjectRoot(element) => element.encode(),
                VmCalls::ReadFile(element) => element.encode(),
                VmCalls::ReadFileBinary(element) => element.encode(),
                VmCalls::ReadLine(element) => element.encode(),
                VmCalls::Record(element) => element.encode(),
                VmCalls::RecordLogs(element) => element.encode(),
                VmCalls::RememberKey(element) => element.encode(),
                VmCalls::RemoveFile(element) => element.encode(),
                VmCalls::RevertTo(element) => element.encode(),
                VmCalls::RevokePersistent0(element) => element.encode(),
                VmCalls::RevokePersistent1(element) => element.encode(),
                VmCalls::Roll(element) => element.encode(),
                VmCalls::RollFork0(element) => element.encode(),
                VmCalls::RollFork2(element) => element.encode(),
                VmCalls::RollFork1(element) => element.encode(),
                VmCalls::RollFork3(element) => element.encode(),
                VmCalls::RpcUrl(element) => element.encode(),
                VmCalls::RpcUrls(element) => element.encode(),
                VmCalls::SelectFork(element) => element.encode(),
                VmCalls::SetEnv(element) => element.encode(),
                VmCalls::SetNonce(element) => element.encode(),
                VmCalls::Sign(element) => element.encode(),
                VmCalls::Snapshot(element) => element.encode(),
                VmCalls::StartBroadcast0(element) => element.encode(),
                VmCalls::StartBroadcast1(element) => element.encode(),
                VmCalls::StartBroadcast2(element) => element.encode(),
                VmCalls::StartPrank0(element) => element.encode(),
                VmCalls::StartPrank1(element) => element.encode(),
                VmCalls::StopBroadcast(element) => element.encode(),
                VmCalls::StopPrank(element) => element.encode(),
                VmCalls::Store(element) => element.encode(),
                VmCalls::ToString0(element) => element.encode(),
                VmCalls::ToString1(element) => element.encode(),
                VmCalls::ToString2(element) => element.encode(),
                VmCalls::ToString3(element) => element.encode(),
                VmCalls::ToString4(element) => element.encode(),
                VmCalls::ToString5(element) => element.encode(),
                VmCalls::TransactWithForkId(element) => element.encode(),
                VmCalls::Transact(element) => element.encode(),
                VmCalls::Warp(element) => element.encode(),
                VmCalls::WriteFile(element) => element.encode(),
                VmCalls::WriteFileBinary(element) => element.encode(),
                VmCalls::WriteLine(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for VmCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                VmCalls::Accesses(element) => element.fmt(f),
                VmCalls::ActiveFork(element) => element.fmt(f),
                VmCalls::Addr(element) => element.fmt(f),
                VmCalls::AllowCheatcodes(element) => element.fmt(f),
                VmCalls::Assume(element) => element.fmt(f),
                VmCalls::Broadcast0(element) => element.fmt(f),
                VmCalls::Broadcast1(element) => element.fmt(f),
                VmCalls::Broadcast2(element) => element.fmt(f),
                VmCalls::ChainId(element) => element.fmt(f),
                VmCalls::ClearMockedCalls(element) => element.fmt(f),
                VmCalls::CloseFile(element) => element.fmt(f),
                VmCalls::Coinbase(element) => element.fmt(f),
                VmCalls::CreateFork0(element) => element.fmt(f),
                VmCalls::CreateFork1(element) => element.fmt(f),
                VmCalls::CreateFork2(element) => element.fmt(f),
                VmCalls::CreateSelectFork1(element) => element.fmt(f),
                VmCalls::CreateSelectFork2(element) => element.fmt(f),
                VmCalls::CreateSelectFork0(element) => element.fmt(f),
                VmCalls::Deal(element) => element.fmt(f),
                VmCalls::DeriveKey0(element) => element.fmt(f),
                VmCalls::DeriveKey1(element) => element.fmt(f),
                VmCalls::Difficulty(element) => element.fmt(f),
                VmCalls::EnvAddress0(element) => element.fmt(f),
                VmCalls::EnvAddress1(element) => element.fmt(f),
                VmCalls::EnvBool0(element) => element.fmt(f),
                VmCalls::EnvBool1(element) => element.fmt(f),
                VmCalls::EnvBytes0(element) => element.fmt(f),
                VmCalls::EnvBytes1(element) => element.fmt(f),
                VmCalls::EnvBytes321(element) => element.fmt(f),
                VmCalls::EnvBytes320(element) => element.fmt(f),
                VmCalls::EnvInt1(element) => element.fmt(f),
                VmCalls::EnvInt0(element) => element.fmt(f),
                VmCalls::EnvString1(element) => element.fmt(f),
                VmCalls::EnvString0(element) => element.fmt(f),
                VmCalls::EnvUint0(element) => element.fmt(f),
                VmCalls::EnvUint1(element) => element.fmt(f),
                VmCalls::Etch(element) => element.fmt(f),
                VmCalls::ExpectCall0(element) => element.fmt(f),
                VmCalls::ExpectCall1(element) => element.fmt(f),
                VmCalls::ExpectEmit0(element) => element.fmt(f),
                VmCalls::ExpectEmit1(element) => element.fmt(f),
                VmCalls::ExpectRevert1(element) => element.fmt(f),
                VmCalls::ExpectRevert2(element) => element.fmt(f),
                VmCalls::ExpectRevert0(element) => element.fmt(f),
                VmCalls::Fee(element) => element.fmt(f),
                VmCalls::Ffi(element) => element.fmt(f),
                VmCalls::GetCode(element) => element.fmt(f),
                VmCalls::GetDeployedCode(element) => element.fmt(f),
                VmCalls::GetNonce(element) => element.fmt(f),
                VmCalls::GetRecordedLogs(element) => element.fmt(f),
                VmCalls::IsPersistent(element) => element.fmt(f),
                VmCalls::Label(element) => element.fmt(f),
                VmCalls::Load(element) => element.fmt(f),
                VmCalls::MakePersistent0(element) => element.fmt(f),
                VmCalls::MakePersistent2(element) => element.fmt(f),
                VmCalls::MakePersistent1(element) => element.fmt(f),
                VmCalls::MakePersistent3(element) => element.fmt(f),
                VmCalls::MockCall1(element) => element.fmt(f),
                VmCalls::MockCall0(element) => element.fmt(f),
                VmCalls::ParseAddress(element) => element.fmt(f),
                VmCalls::ParseBool(element) => element.fmt(f),
                VmCalls::ParseBytes(element) => element.fmt(f),
                VmCalls::ParseBytes32(element) => element.fmt(f),
                VmCalls::ParseInt(element) => element.fmt(f),
                VmCalls::ParseJson0(element) => element.fmt(f),
                VmCalls::ParseJson1(element) => element.fmt(f),
                VmCalls::ParseUint(element) => element.fmt(f),
                VmCalls::Prank1(element) => element.fmt(f),
                VmCalls::Prank0(element) => element.fmt(f),
                VmCalls::ProjectRoot(element) => element.fmt(f),
                VmCalls::ReadFile(element) => element.fmt(f),
                VmCalls::ReadFileBinary(element) => element.fmt(f),
                VmCalls::ReadLine(element) => element.fmt(f),
                VmCalls::Record(element) => element.fmt(f),
                VmCalls::RecordLogs(element) => element.fmt(f),
                VmCalls::RememberKey(element) => element.fmt(f),
                VmCalls::RemoveFile(element) => element.fmt(f),
                VmCalls::RevertTo(element) => element.fmt(f),
                VmCalls::RevokePersistent0(element) => element.fmt(f),
                VmCalls::RevokePersistent1(element) => element.fmt(f),
                VmCalls::Roll(element) => element.fmt(f),
                VmCalls::RollFork0(element) => element.fmt(f),
                VmCalls::RollFork2(element) => element.fmt(f),
                VmCalls::RollFork1(element) => element.fmt(f),
                VmCalls::RollFork3(element) => element.fmt(f),
                VmCalls::RpcUrl(element) => element.fmt(f),
                VmCalls::RpcUrls(element) => element.fmt(f),
                VmCalls::SelectFork(element) => element.fmt(f),
                VmCalls::SetEnv(element) => element.fmt(f),
                VmCalls::SetNonce(element) => element.fmt(f),
                VmCalls::Sign(element) => element.fmt(f),
                VmCalls::Snapshot(element) => element.fmt(f),
                VmCalls::StartBroadcast0(element) => element.fmt(f),
                VmCalls::StartBroadcast1(element) => element.fmt(f),
                VmCalls::StartBroadcast2(element) => element.fmt(f),
                VmCalls::StartPrank0(element) => element.fmt(f),
                VmCalls::StartPrank1(element) => element.fmt(f),
                VmCalls::StopBroadcast(element) => element.fmt(f),
                VmCalls::StopPrank(element) => element.fmt(f),
                VmCalls::Store(element) => element.fmt(f),
                VmCalls::ToString0(element) => element.fmt(f),
                VmCalls::ToString1(element) => element.fmt(f),
                VmCalls::ToString2(element) => element.fmt(f),
                VmCalls::ToString3(element) => element.fmt(f),
                VmCalls::ToString4(element) => element.fmt(f),
                VmCalls::ToString5(element) => element.fmt(f),
                VmCalls::TransactWithForkId(element) => element.fmt(f),
                VmCalls::Transact(element) => element.fmt(f),
                VmCalls::Warp(element) => element.fmt(f),
                VmCalls::WriteFile(element) => element.fmt(f),
                VmCalls::WriteFileBinary(element) => element.fmt(f),
                VmCalls::WriteLine(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AccessesCall> for VmCalls {
        fn from(var: AccessesCall) -> Self {
            VmCalls::Accesses(var)
        }
    }
    impl ::std::convert::From<ActiveForkCall> for VmCalls {
        fn from(var: ActiveForkCall) -> Self {
            VmCalls::ActiveFork(var)
        }
    }
    impl ::std::convert::From<AddrCall> for VmCalls {
        fn from(var: AddrCall) -> Self {
            VmCalls::Addr(var)
        }
    }
    impl ::std::convert::From<AllowCheatcodesCall> for VmCalls {
        fn from(var: AllowCheatcodesCall) -> Self {
            VmCalls::AllowCheatcodes(var)
        }
    }
    impl ::std::convert::From<AssumeCall> for VmCalls {
        fn from(var: AssumeCall) -> Self {
            VmCalls::Assume(var)
        }
    }
    impl ::std::convert::From<Broadcast0Call> for VmCalls {
        fn from(var: Broadcast0Call) -> Self {
            VmCalls::Broadcast0(var)
        }
    }
    impl ::std::convert::From<Broadcast1Call> for VmCalls {
        fn from(var: Broadcast1Call) -> Self {
            VmCalls::Broadcast1(var)
        }
    }
    impl ::std::convert::From<Broadcast2Call> for VmCalls {
        fn from(var: Broadcast2Call) -> Self {
            VmCalls::Broadcast2(var)
        }
    }
    impl ::std::convert::From<ChainIdCall> for VmCalls {
        fn from(var: ChainIdCall) -> Self {
            VmCalls::ChainId(var)
        }
    }
    impl ::std::convert::From<ClearMockedCallsCall> for VmCalls {
        fn from(var: ClearMockedCallsCall) -> Self {
            VmCalls::ClearMockedCalls(var)
        }
    }
    impl ::std::convert::From<CloseFileCall> for VmCalls {
        fn from(var: CloseFileCall) -> Self {
            VmCalls::CloseFile(var)
        }
    }
    impl ::std::convert::From<CoinbaseCall> for VmCalls {
        fn from(var: CoinbaseCall) -> Self {
            VmCalls::Coinbase(var)
        }
    }
    impl ::std::convert::From<CreateFork0Call> for VmCalls {
        fn from(var: CreateFork0Call) -> Self {
            VmCalls::CreateFork0(var)
        }
    }
    impl ::std::convert::From<CreateFork1Call> for VmCalls {
        fn from(var: CreateFork1Call) -> Self {
            VmCalls::CreateFork1(var)
        }
    }
    impl ::std::convert::From<CreateFork2Call> for VmCalls {
        fn from(var: CreateFork2Call) -> Self {
            VmCalls::CreateFork2(var)
        }
    }
    impl ::std::convert::From<CreateSelectFork1Call> for VmCalls {
        fn from(var: CreateSelectFork1Call) -> Self {
            VmCalls::CreateSelectFork1(var)
        }
    }
    impl ::std::convert::From<CreateSelectFork2Call> for VmCalls {
        fn from(var: CreateSelectFork2Call) -> Self {
            VmCalls::CreateSelectFork2(var)
        }
    }
    impl ::std::convert::From<CreateSelectFork0Call> for VmCalls {
        fn from(var: CreateSelectFork0Call) -> Self {
            VmCalls::CreateSelectFork0(var)
        }
    }
    impl ::std::convert::From<DealCall> for VmCalls {
        fn from(var: DealCall) -> Self {
            VmCalls::Deal(var)
        }
    }
    impl ::std::convert::From<DeriveKey0Call> for VmCalls {
        fn from(var: DeriveKey0Call) -> Self {
            VmCalls::DeriveKey0(var)
        }
    }
    impl ::std::convert::From<DeriveKey1Call> for VmCalls {
        fn from(var: DeriveKey1Call) -> Self {
            VmCalls::DeriveKey1(var)
        }
    }
    impl ::std::convert::From<DifficultyCall> for VmCalls {
        fn from(var: DifficultyCall) -> Self {
            VmCalls::Difficulty(var)
        }
    }
    impl ::std::convert::From<EnvAddress0Call> for VmCalls {
        fn from(var: EnvAddress0Call) -> Self {
            VmCalls::EnvAddress0(var)
        }
    }
    impl ::std::convert::From<EnvAddress1Call> for VmCalls {
        fn from(var: EnvAddress1Call) -> Self {
            VmCalls::EnvAddress1(var)
        }
    }
    impl ::std::convert::From<EnvBool0Call> for VmCalls {
        fn from(var: EnvBool0Call) -> Self {
            VmCalls::EnvBool0(var)
        }
    }
    impl ::std::convert::From<EnvBool1Call> for VmCalls {
        fn from(var: EnvBool1Call) -> Self {
            VmCalls::EnvBool1(var)
        }
    }
    impl ::std::convert::From<EnvBytes0Call> for VmCalls {
        fn from(var: EnvBytes0Call) -> Self {
            VmCalls::EnvBytes0(var)
        }
    }
    impl ::std::convert::From<EnvBytes1Call> for VmCalls {
        fn from(var: EnvBytes1Call) -> Self {
            VmCalls::EnvBytes1(var)
        }
    }
    impl ::std::convert::From<EnvBytes321Call> for VmCalls {
        fn from(var: EnvBytes321Call) -> Self {
            VmCalls::EnvBytes321(var)
        }
    }
    impl ::std::convert::From<EnvBytes320Call> for VmCalls {
        fn from(var: EnvBytes320Call) -> Self {
            VmCalls::EnvBytes320(var)
        }
    }
    impl ::std::convert::From<EnvInt1Call> for VmCalls {
        fn from(var: EnvInt1Call) -> Self {
            VmCalls::EnvInt1(var)
        }
    }
    impl ::std::convert::From<EnvInt0Call> for VmCalls {
        fn from(var: EnvInt0Call) -> Self {
            VmCalls::EnvInt0(var)
        }
    }
    impl ::std::convert::From<EnvString1Call> for VmCalls {
        fn from(var: EnvString1Call) -> Self {
            VmCalls::EnvString1(var)
        }
    }
    impl ::std::convert::From<EnvString0Call> for VmCalls {
        fn from(var: EnvString0Call) -> Self {
            VmCalls::EnvString0(var)
        }
    }
    impl ::std::convert::From<EnvUint0Call> for VmCalls {
        fn from(var: EnvUint0Call) -> Self {
            VmCalls::EnvUint0(var)
        }
    }
    impl ::std::convert::From<EnvUint1Call> for VmCalls {
        fn from(var: EnvUint1Call) -> Self {
            VmCalls::EnvUint1(var)
        }
    }
    impl ::std::convert::From<EtchCall> for VmCalls {
        fn from(var: EtchCall) -> Self {
            VmCalls::Etch(var)
        }
    }
    impl ::std::convert::From<ExpectCall0Call> for VmCalls {
        fn from(var: ExpectCall0Call) -> Self {
            VmCalls::ExpectCall0(var)
        }
    }
    impl ::std::convert::From<ExpectCall1Call> for VmCalls {
        fn from(var: ExpectCall1Call) -> Self {
            VmCalls::ExpectCall1(var)
        }
    }
    impl ::std::convert::From<ExpectEmit0Call> for VmCalls {
        fn from(var: ExpectEmit0Call) -> Self {
            VmCalls::ExpectEmit0(var)
        }
    }
    impl ::std::convert::From<ExpectEmit1Call> for VmCalls {
        fn from(var: ExpectEmit1Call) -> Self {
            VmCalls::ExpectEmit1(var)
        }
    }
    impl ::std::convert::From<ExpectRevert1Call> for VmCalls {
        fn from(var: ExpectRevert1Call) -> Self {
            VmCalls::ExpectRevert1(var)
        }
    }
    impl ::std::convert::From<ExpectRevert2Call> for VmCalls {
        fn from(var: ExpectRevert2Call) -> Self {
            VmCalls::ExpectRevert2(var)
        }
    }
    impl ::std::convert::From<ExpectRevert0Call> for VmCalls {
        fn from(var: ExpectRevert0Call) -> Self {
            VmCalls::ExpectRevert0(var)
        }
    }
    impl ::std::convert::From<FeeCall> for VmCalls {
        fn from(var: FeeCall) -> Self {
            VmCalls::Fee(var)
        }
    }
    impl ::std::convert::From<FfiCall> for VmCalls {
        fn from(var: FfiCall) -> Self {
            VmCalls::Ffi(var)
        }
    }
    impl ::std::convert::From<GetCodeCall> for VmCalls {
        fn from(var: GetCodeCall) -> Self {
            VmCalls::GetCode(var)
        }
    }
    impl ::std::convert::From<GetDeployedCodeCall> for VmCalls {
        fn from(var: GetDeployedCodeCall) -> Self {
            VmCalls::GetDeployedCode(var)
        }
    }
    impl ::std::convert::From<GetNonceCall> for VmCalls {
        fn from(var: GetNonceCall) -> Self {
            VmCalls::GetNonce(var)
        }
    }
    impl ::std::convert::From<GetRecordedLogsCall> for VmCalls {
        fn from(var: GetRecordedLogsCall) -> Self {
            VmCalls::GetRecordedLogs(var)
        }
    }
    impl ::std::convert::From<IsPersistentCall> for VmCalls {
        fn from(var: IsPersistentCall) -> Self {
            VmCalls::IsPersistent(var)
        }
    }
    impl ::std::convert::From<LabelCall> for VmCalls {
        fn from(var: LabelCall) -> Self {
            VmCalls::Label(var)
        }
    }
    impl ::std::convert::From<LoadCall> for VmCalls {
        fn from(var: LoadCall) -> Self {
            VmCalls::Load(var)
        }
    }
    impl ::std::convert::From<MakePersistent0Call> for VmCalls {
        fn from(var: MakePersistent0Call) -> Self {
            VmCalls::MakePersistent0(var)
        }
    }
    impl ::std::convert::From<MakePersistent2Call> for VmCalls {
        fn from(var: MakePersistent2Call) -> Self {
            VmCalls::MakePersistent2(var)
        }
    }
    impl ::std::convert::From<MakePersistent1Call> for VmCalls {
        fn from(var: MakePersistent1Call) -> Self {
            VmCalls::MakePersistent1(var)
        }
    }
    impl ::std::convert::From<MakePersistent3Call> for VmCalls {
        fn from(var: MakePersistent3Call) -> Self {
            VmCalls::MakePersistent3(var)
        }
    }
    impl ::std::convert::From<MockCall1Call> for VmCalls {
        fn from(var: MockCall1Call) -> Self {
            VmCalls::MockCall1(var)
        }
    }
    impl ::std::convert::From<MockCall0Call> for VmCalls {
        fn from(var: MockCall0Call) -> Self {
            VmCalls::MockCall0(var)
        }
    }
    impl ::std::convert::From<ParseAddressCall> for VmCalls {
        fn from(var: ParseAddressCall) -> Self {
            VmCalls::ParseAddress(var)
        }
    }
    impl ::std::convert::From<ParseBoolCall> for VmCalls {
        fn from(var: ParseBoolCall) -> Self {
            VmCalls::ParseBool(var)
        }
    }
    impl ::std::convert::From<ParseBytesCall> for VmCalls {
        fn from(var: ParseBytesCall) -> Self {
            VmCalls::ParseBytes(var)
        }
    }
    impl ::std::convert::From<ParseBytes32Call> for VmCalls {
        fn from(var: ParseBytes32Call) -> Self {
            VmCalls::ParseBytes32(var)
        }
    }
    impl ::std::convert::From<ParseIntCall> for VmCalls {
        fn from(var: ParseIntCall) -> Self {
            VmCalls::ParseInt(var)
        }
    }
    impl ::std::convert::From<ParseJson0Call> for VmCalls {
        fn from(var: ParseJson0Call) -> Self {
            VmCalls::ParseJson0(var)
        }
    }
    impl ::std::convert::From<ParseJson1Call> for VmCalls {
        fn from(var: ParseJson1Call) -> Self {
            VmCalls::ParseJson1(var)
        }
    }
    impl ::std::convert::From<ParseUintCall> for VmCalls {
        fn from(var: ParseUintCall) -> Self {
            VmCalls::ParseUint(var)
        }
    }
    impl ::std::convert::From<Prank1Call> for VmCalls {
        fn from(var: Prank1Call) -> Self {
            VmCalls::Prank1(var)
        }
    }
    impl ::std::convert::From<Prank0Call> for VmCalls {
        fn from(var: Prank0Call) -> Self {
            VmCalls::Prank0(var)
        }
    }
    impl ::std::convert::From<ProjectRootCall> for VmCalls {
        fn from(var: ProjectRootCall) -> Self {
            VmCalls::ProjectRoot(var)
        }
    }
    impl ::std::convert::From<ReadFileCall> for VmCalls {
        fn from(var: ReadFileCall) -> Self {
            VmCalls::ReadFile(var)
        }
    }
    impl ::std::convert::From<ReadFileBinaryCall> for VmCalls {
        fn from(var: ReadFileBinaryCall) -> Self {
            VmCalls::ReadFileBinary(var)
        }
    }
    impl ::std::convert::From<ReadLineCall> for VmCalls {
        fn from(var: ReadLineCall) -> Self {
            VmCalls::ReadLine(var)
        }
    }
    impl ::std::convert::From<RecordCall> for VmCalls {
        fn from(var: RecordCall) -> Self {
            VmCalls::Record(var)
        }
    }
    impl ::std::convert::From<RecordLogsCall> for VmCalls {
        fn from(var: RecordLogsCall) -> Self {
            VmCalls::RecordLogs(var)
        }
    }
    impl ::std::convert::From<RememberKeyCall> for VmCalls {
        fn from(var: RememberKeyCall) -> Self {
            VmCalls::RememberKey(var)
        }
    }
    impl ::std::convert::From<RemoveFileCall> for VmCalls {
        fn from(var: RemoveFileCall) -> Self {
            VmCalls::RemoveFile(var)
        }
    }
    impl ::std::convert::From<RevertToCall> for VmCalls {
        fn from(var: RevertToCall) -> Self {
            VmCalls::RevertTo(var)
        }
    }
    impl ::std::convert::From<RevokePersistent0Call> for VmCalls {
        fn from(var: RevokePersistent0Call) -> Self {
            VmCalls::RevokePersistent0(var)
        }
    }
    impl ::std::convert::From<RevokePersistent1Call> for VmCalls {
        fn from(var: RevokePersistent1Call) -> Self {
            VmCalls::RevokePersistent1(var)
        }
    }
    impl ::std::convert::From<RollCall> for VmCalls {
        fn from(var: RollCall) -> Self {
            VmCalls::Roll(var)
        }
    }
    impl ::std::convert::From<RollFork0Call> for VmCalls {
        fn from(var: RollFork0Call) -> Self {
            VmCalls::RollFork0(var)
        }
    }
    impl ::std::convert::From<RollFork2Call> for VmCalls {
        fn from(var: RollFork2Call) -> Self {
            VmCalls::RollFork2(var)
        }
    }
    impl ::std::convert::From<RollFork1Call> for VmCalls {
        fn from(var: RollFork1Call) -> Self {
            VmCalls::RollFork1(var)
        }
    }
    impl ::std::convert::From<RollFork3Call> for VmCalls {
        fn from(var: RollFork3Call) -> Self {
            VmCalls::RollFork3(var)
        }
    }
    impl ::std::convert::From<RpcUrlCall> for VmCalls {
        fn from(var: RpcUrlCall) -> Self {
            VmCalls::RpcUrl(var)
        }
    }
    impl ::std::convert::From<RpcUrlsCall> for VmCalls {
        fn from(var: RpcUrlsCall) -> Self {
            VmCalls::RpcUrls(var)
        }
    }
    impl ::std::convert::From<SelectForkCall> for VmCalls {
        fn from(var: SelectForkCall) -> Self {
            VmCalls::SelectFork(var)
        }
    }
    impl ::std::convert::From<SetEnvCall> for VmCalls {
        fn from(var: SetEnvCall) -> Self {
            VmCalls::SetEnv(var)
        }
    }
    impl ::std::convert::From<SetNonceCall> for VmCalls {
        fn from(var: SetNonceCall) -> Self {
            VmCalls::SetNonce(var)
        }
    }
    impl ::std::convert::From<SignCall> for VmCalls {
        fn from(var: SignCall) -> Self {
            VmCalls::Sign(var)
        }
    }
    impl ::std::convert::From<SnapshotCall> for VmCalls {
        fn from(var: SnapshotCall) -> Self {
            VmCalls::Snapshot(var)
        }
    }
    impl ::std::convert::From<StartBroadcast0Call> for VmCalls {
        fn from(var: StartBroadcast0Call) -> Self {
            VmCalls::StartBroadcast0(var)
        }
    }
    impl ::std::convert::From<StartBroadcast1Call> for VmCalls {
        fn from(var: StartBroadcast1Call) -> Self {
            VmCalls::StartBroadcast1(var)
        }
    }
    impl ::std::convert::From<StartBroadcast2Call> for VmCalls {
        fn from(var: StartBroadcast2Call) -> Self {
            VmCalls::StartBroadcast2(var)
        }
    }
    impl ::std::convert::From<StartPrank0Call> for VmCalls {
        fn from(var: StartPrank0Call) -> Self {
            VmCalls::StartPrank0(var)
        }
    }
    impl ::std::convert::From<StartPrank1Call> for VmCalls {
        fn from(var: StartPrank1Call) -> Self {
            VmCalls::StartPrank1(var)
        }
    }
    impl ::std::convert::From<StopBroadcastCall> for VmCalls {
        fn from(var: StopBroadcastCall) -> Self {
            VmCalls::StopBroadcast(var)
        }
    }
    impl ::std::convert::From<StopPrankCall> for VmCalls {
        fn from(var: StopPrankCall) -> Self {
            VmCalls::StopPrank(var)
        }
    }
    impl ::std::convert::From<StoreCall> for VmCalls {
        fn from(var: StoreCall) -> Self {
            VmCalls::Store(var)
        }
    }
    impl ::std::convert::From<ToString0Call> for VmCalls {
        fn from(var: ToString0Call) -> Self {
            VmCalls::ToString0(var)
        }
    }
    impl ::std::convert::From<ToString1Call> for VmCalls {
        fn from(var: ToString1Call) -> Self {
            VmCalls::ToString1(var)
        }
    }
    impl ::std::convert::From<ToString2Call> for VmCalls {
        fn from(var: ToString2Call) -> Self {
            VmCalls::ToString2(var)
        }
    }
    impl ::std::convert::From<ToString3Call> for VmCalls {
        fn from(var: ToString3Call) -> Self {
            VmCalls::ToString3(var)
        }
    }
    impl ::std::convert::From<ToString4Call> for VmCalls {
        fn from(var: ToString4Call) -> Self {
            VmCalls::ToString4(var)
        }
    }
    impl ::std::convert::From<ToString5Call> for VmCalls {
        fn from(var: ToString5Call) -> Self {
            VmCalls::ToString5(var)
        }
    }
    impl ::std::convert::From<TransactWithForkIdCall> for VmCalls {
        fn from(var: TransactWithForkIdCall) -> Self {
            VmCalls::TransactWithForkId(var)
        }
    }
    impl ::std::convert::From<TransactCall> for VmCalls {
        fn from(var: TransactCall) -> Self {
            VmCalls::Transact(var)
        }
    }
    impl ::std::convert::From<WarpCall> for VmCalls {
        fn from(var: WarpCall) -> Self {
            VmCalls::Warp(var)
        }
    }
    impl ::std::convert::From<WriteFileCall> for VmCalls {
        fn from(var: WriteFileCall) -> Self {
            VmCalls::WriteFile(var)
        }
    }
    impl ::std::convert::From<WriteFileBinaryCall> for VmCalls {
        fn from(var: WriteFileBinaryCall) -> Self {
            VmCalls::WriteFileBinary(var)
        }
    }
    impl ::std::convert::From<WriteLineCall> for VmCalls {
        fn from(var: WriteLineCall) -> Self {
            VmCalls::WriteLine(var)
        }
    }
    #[doc = "Container type for all return fields from the `accesses` function with signature `accesses(address)` and selector `[101, 188, 148, 129]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AccessesReturn {
        pub reads: ::std::vec::Vec<[u8; 32]>,
        pub writes: ::std::vec::Vec<[u8; 32]>,
    }
    #[doc = "Container type for all return fields from the `activeFork` function with signature `activeFork()` and selector `[47, 16, 63, 34]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ActiveForkReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `addr` function with signature `addr(uint256)` and selector `[255, 161, 134, 73]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AddrReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `createFork` function with signature `createFork(string)` and selector `[49, 186, 52, 152]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CreateFork0Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `createFork` function with signature `createFork(string,uint256)` and selector `[107, 163, 186, 43]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CreateFork1Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `createFork` function with signature `createFork(string,bytes32)` and selector `[124, 162, 150, 130]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CreateFork2Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `createSelectFork` function with signature `createSelectFork(string,uint256)` and selector `[113, 238, 70, 77]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CreateSelectFork1Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `createSelectFork` function with signature `createSelectFork(string,bytes32)` and selector `[132, 213, 43, 122]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CreateSelectFork2Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `createSelectFork` function with signature `createSelectFork(string)` and selector `[152, 104, 0, 52]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CreateSelectFork0Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `deriveKey` function with signature `deriveKey(string,uint32)` and selector `[98, 41, 73, 139]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DeriveKey0Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `deriveKey` function with signature `deriveKey(string,string,uint32)` and selector `[107, 203, 44, 27]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DeriveKey1Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `envAddress` function with signature `envAddress(string)` and selector `[53, 13, 86, 191]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvAddress0Return(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `envAddress` function with signature `envAddress(string,string)` and selector `[173, 49, 185, 250]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvAddress1Return(pub ::std::vec::Vec<ethers::core::types::Address>);
    #[doc = "Container type for all return fields from the `envBool` function with signature `envBool(string)` and selector `[126, 209, 236, 125]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvBool0Return(pub bool);
    #[doc = "Container type for all return fields from the `envBool` function with signature `envBool(string,string)` and selector `[170, 173, 222, 175]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvBool1Return(pub ::std::vec::Vec<bool>);
    #[doc = "Container type for all return fields from the `envBytes` function with signature `envBytes(string)` and selector `[77, 123, 175, 6]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvBytes0Return(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `envBytes` function with signature `envBytes(string,string)` and selector `[221, 194, 101, 27]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvBytes1Return(pub ::std::vec::Vec<ethers::core::types::Bytes>);
    #[doc = "Container type for all return fields from the `envBytes32` function with signature `envBytes32(string,string)` and selector `[90, 242, 49, 193]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvBytes321Return(pub ::std::vec::Vec<[u8; 32]>);
    #[doc = "Container type for all return fields from the `envBytes32` function with signature `envBytes32(string)` and selector `[151, 148, 144, 66]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvBytes320Return(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `envInt` function with signature `envInt(string,string)` and selector `[66, 24, 17, 80]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvInt1Return(pub ::std::vec::Vec<I256>);
    #[doc = "Container type for all return fields from the `envInt` function with signature `envInt(string)` and selector `[137, 42, 12, 97]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvInt0Return(pub I256);
    #[doc = "Container type for all return fields from the `envString` function with signature `envString(string,string)` and selector `[20, 176, 43, 201]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvString1Return(pub ::std::vec::Vec<String>);
    #[doc = "Container type for all return fields from the `envString` function with signature `envString(string)` and selector `[248, 119, 203, 25]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvString0Return(pub String);
    #[doc = "Container type for all return fields from the `envUint` function with signature `envUint(string)` and selector `[193, 151, 141, 31]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvUint0Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `envUint` function with signature `envUint(string,string)` and selector `[243, 222, 192, 153]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnvUint1Return(pub ::std::vec::Vec<ethers::core::types::U256>);
    #[doc = "Container type for all return fields from the `ffi` function with signature `ffi(string[])` and selector `[137, 22, 4, 103]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FfiReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `getCode` function with signature `getCode(string)` and selector `[141, 28, 201, 37]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetCodeReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `getDeployedCode` function with signature `getDeployedCode(string)` and selector `[62, 191, 115, 180]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetDeployedCodeReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `getNonce` function with signature `getNonce(address)` and selector `[45, 3, 53, 171]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetNonceReturn(pub u64);
    #[doc = "Container type for all return fields from the `getRecordedLogs` function with signature `getRecordedLogs()` and selector `[25, 21, 83, 164]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetRecordedLogsReturn(pub ::std::vec::Vec<Log>);
    #[doc = "Container type for all return fields from the `isPersistent` function with signature `isPersistent(address)` and selector `[217, 45, 142, 253]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsPersistentReturn(pub bool);
    #[doc = "Container type for all return fields from the `load` function with signature `load(address,bytes32)` and selector `[102, 127, 157, 112]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct LoadReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `parseAddress` function with signature `parseAddress(string)` and selector `[198, 206, 5, 157]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ParseAddressReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `parseBool` function with signature `parseBool(string)` and selector `[151, 78, 249, 36]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ParseBoolReturn(pub bool);
    #[doc = "Container type for all return fields from the `parseBytes` function with signature `parseBytes(string)` and selector `[143, 93, 35, 45]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ParseBytesReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `parseBytes32` function with signature `parseBytes32(string)` and selector `[8, 126, 110, 129]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ParseBytes32Return(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `parseInt` function with signature `parseInt(string)` and selector `[66, 52, 108, 94]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ParseIntReturn(pub I256);
    #[doc = "Container type for all return fields from the `parseJson` function with signature `parseJson(string)` and selector `[106, 130, 96, 10]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ParseJson0Return(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `parseJson` function with signature `parseJson(string,string)` and selector `[133, 148, 14, 241]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ParseJson1Return(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `parseUint` function with signature `parseUint(string)` and selector `[250, 145, 69, 77]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ParseUintReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `projectRoot` function with signature `projectRoot()` and selector `[217, 48, 160, 230]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ProjectRootReturn(pub String);
    #[doc = "Container type for all return fields from the `readFile` function with signature `readFile(string)` and selector `[96, 249, 187, 17]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ReadFileReturn(pub String);
    #[doc = "Container type for all return fields from the `readFileBinary` function with signature `readFileBinary(string)` and selector `[22, 237, 123, 196]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ReadFileBinaryReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `readLine` function with signature `readLine(string)` and selector `[112, 245, 87, 40]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ReadLineReturn(pub String);
    #[doc = "Container type for all return fields from the `rememberKey` function with signature `rememberKey(uint256)` and selector `[34, 16, 0, 100]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RememberKeyReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `revertTo` function with signature `revertTo(uint256)` and selector `[68, 215, 240, 164]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RevertToReturn(pub bool);
    #[doc = "Container type for all return fields from the `rpcUrl` function with signature `rpcUrl(string)` and selector `[151, 90, 108, 233]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RpcUrlReturn(pub String);
    #[doc = "Container type for all return fields from the `rpcUrls` function with signature `rpcUrls()` and selector `[168, 90, 132, 24]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RpcUrlsReturn(pub ::std::vec::Vec<[String; 2usize]>);
    #[doc = "Container type for all return fields from the `sign` function with signature `sign(uint256,bytes32)` and selector `[227, 65, 234, 164]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SignReturn(pub u8, pub [u8; 32], pub [u8; 32]);
    #[doc = "Container type for all return fields from the `snapshot` function with signature `snapshot()` and selector `[151, 17, 113, 90]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SnapshotReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `toString` function with signature `toString(address)` and selector `[86, 202, 98, 62]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ToString0Return(pub String);
    #[doc = "Container type for all return fields from the `toString` function with signature `toString(uint256)` and selector `[105, 0, 163, 174]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ToString1Return(pub String);
    #[doc = "Container type for all return fields from the `toString` function with signature `toString(bytes)` and selector `[113, 170, 209, 13]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ToString2Return(pub String);
    #[doc = "Container type for all return fields from the `toString` function with signature `toString(bool)` and selector `[113, 220, 231, 218]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ToString3Return(pub String);
    #[doc = "Container type for all return fields from the `toString` function with signature `toString(int256)` and selector `[163, 34, 196, 14]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ToString4Return(pub String);
    #[doc = "Container type for all return fields from the `toString` function with signature `toString(bytes32)` and selector `[177, 26, 25, 232]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ToString5Return(pub String);
}
