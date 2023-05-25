// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity >=0.8.0;

contract TestPluckParam {
    event Dynamic32(uint256[] first);
    event Static(bytes4 first);
    event StaticDynamic(bytes4 first, string second);
    event StaticDynamicDynamic32(address first, bytes second, uint32[] third);
    event StaticDynamic32Dynamic(uint32 first, bytes4[] second, string third);
    event DynamicStaticDynamic32(bytes first, bool second, bytes2[] third);
    event DynamicDynamic32Static(string first, uint32[] second, uint256 third);
    event Dynamic32StaticDynamic(address[] first, bytes2 second, bytes third);
    event Dynamic32DynamicStatic(bytes2[] first, string second, uint32 third);
    event UnsupportedFixedSizeAndDynamic(bool[2] first, string second);

    function staticFn(bytes4 first) external {
        emit Static(first);
    }

    function staticDynamic(bytes4 first, string memory second) external {
        emit StaticDynamic(first, second);
    }

    function staticDynamicDynamic32(
        address first,
        bytes calldata second,
        uint32[] memory third
    ) external {
        emit StaticDynamicDynamic32(first, second, third);
    }

    function staticDynamic32Dynamic(
        uint32 first,
        bytes4[] calldata second,
        string memory third
    ) external {
        emit StaticDynamic32Dynamic(first, second, third);
    }

    function dynamicStaticDynamic32(
        bytes calldata first,
        bool second,
        bytes2[] memory third
    ) external {
        emit DynamicStaticDynamic32(first, second, third);
    }

    function dynamicDynamic32Static(
        string calldata first,
        uint32[] memory second,
        uint256 third
    ) external {
        emit DynamicDynamic32Static(first, second, third);
    }

    function dynamic32StaticDynamic(
        address[] calldata first,
        bytes2 second,
        bytes memory third
    ) external {
        emit Dynamic32StaticDynamic(first, second, third);
    }

    function dynamic32DynamicStatic(
        bytes2[] calldata first,
        string memory second,
        uint32 third
    ) external {
        emit Dynamic32DynamicStatic(first, second, third);
    }

    function unsupportedFixedSizeAndDynamic(
        bool[2] memory first,
        string memory second
    ) external {
        emit UnsupportedFixedSizeAndDynamic(first, second);
    }
}
