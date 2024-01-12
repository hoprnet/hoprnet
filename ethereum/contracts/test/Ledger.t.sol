// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { Test, console2 } from "forge-std/Test.sol";
import { HoprChannelsEvents } from "../src/Channels.sol";
import { HoprLedgerEvents, HoprLedger } from "../src/Ledger.sol";
import { CryptoUtils } from "./utils/Crypto.sol";

uint256 constant ONE_HOUR = 60 * 60 * 1000; // in milliseconds

uint256 constant INDEX_SNAPSHOT_INTERVAL = ONE_HOUR;

// proxy contract to make modifiers testable and manipulate storage
contract HoprLedgerTest is Test, HoprLedger(INDEX_SNAPSHOT_INTERVAL), HoprChannelsEvents {
    function setUp() public { }

    function test_update_domain_separator(uint256 newChainId) public {
        newChainId = bound(newChainId, 1, 1e18);
        vm.assume(newChainId != block.chainid);
        bytes32 domainSeparatorOnDeployment = ledgerDomainSeparator;

        // call updateDomainSeparator when chainid is the same
        updateLedgerDomainSeparator();
        assertEq(ledgerDomainSeparator, domainSeparatorOnDeployment);

        // call updateDomainSeparator when chainid is different
        vm.chainId(newChainId);
        vm.expectEmit(true, true, false, false, address(this));
        emit LedgerDomainSeparatorUpdated(
            keccak256(
                abi.encode(
                    keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                    keccak256(bytes("HoprLedger")),
                    keccak256(bytes(LEDGER_VERSION)),
                    newChainId,
                    address(this)
                )
            )
        );
        updateLedgerDomainSeparator();
        assertTrue(ledgerDomainSeparator != domainSeparatorOnDeployment);
    }

    function test_initialState() public {
        assertEq(latestRoot.timestamp, block.timestamp);
        assertEq(latestSnapshotRoot.timestamp, block.timestamp);

        assertEq(latestRoot.rootHash, latestSnapshotRoot.rootHash);
    }

    function test_indexing(uint32 firstTimestamp, uint32 secondTimestamp) public {
        bytes28 initialRoot = latestSnapshotRoot.rootHash;
        uint32 initialTimestamp = latestSnapshotRoot.timestamp;

        firstTimestamp =
            uint32(bound(firstTimestamp, latestRoot.timestamp, INDEX_SNAPSHOT_INTERVAL - latestRoot.timestamp));
        vm.warp(firstTimestamp);

        bytes28 currentRootHash = latestRoot.rootHash;
        uint32 currentBlockNumber = uint32(block.number);

        indexEvent(abi.encodePacked(ChannelOpened.selector));

        // snapshot should be unchanged
        assertEq(initialRoot, latestSnapshotRoot.rootHash);
        assertEq(initialTimestamp, latestSnapshotRoot.timestamp);

        // check new root
        assertEq(
            latestRoot.rootHash,
            bytes28(
                keccak256(
                    abi.encodePacked(
                        // ledger feed must be unique
                        ledgerDomainSeparator,
                        // Allows the verifier to detect up until which block the snapshot includes state changes
                        currentBlockNumber,
                        // Bind result to previous root
                        currentRootHash,
                        // Information about the happened state change
                        keccak256(abi.encodePacked(ChannelOpened.selector))
                    )
                )
            )
        );
        assertEq(latestRoot.timestamp, firstTimestamp);

        // currentRootHash = latestRoot.rootHash;

        vm.roll(currentBlockNumber + 1);

        indexEvent(abi.encodePacked(ChannelOpened.selector));

        // test chainability
        assertEq(
            latestRoot.rootHash,
            bytes28(
                keccak256(
                    abi.encodePacked(
                        // ledger feed must be unique
                        ledgerDomainSeparator,
                        // Allows the verifier to detect up until which block the snapshot includes state changes
                        currentBlockNumber + 1,
                        // Bind result to previous root
                        bytes28(
                            keccak256(
                                abi.encodePacked(
                                    // ledger feed must be unique
                                    ledgerDomainSeparator,
                                    // Allows the verifier to detect up until which block the snapshot includes state
                                    // changes
                                    currentBlockNumber,
                                    // Bind result to previous root
                                    currentRootHash,
                                    // Information about the happened state change
                                    keccak256(abi.encodePacked(ChannelOpened.selector))
                                )
                            )
                        ),
                        // Information about the happened state change
                        keccak256(abi.encodePacked(ChannelOpened.selector))
                    )
                )
            )
        );

        // snapshot should be unchanged
        assertEq(initialRoot, latestSnapshotRoot.rootHash);
        assertEq(initialTimestamp, latestSnapshotRoot.timestamp);

        secondTimestamp =
            uint32(bound(secondTimestamp, latestRoot.timestamp + INDEX_SNAPSHOT_INTERVAL + 1, type(uint32).max));
        vm.warp(secondTimestamp);

        uint32 newBlockNumber = uint32(block.number);

        indexEvent(abi.encodePacked(ChannelOpened.selector));

        assertTrue(latestSnapshotRoot.rootHash != initialRoot);
        assertEq(latestRoot.rootHash, latestSnapshotRoot.rootHash);
        assertEq(latestRoot.timestamp, latestSnapshotRoot.timestamp);

        // test chainability
        assertEq(
            latestRoot.rootHash,
            bytes28(
                keccak256(
                    abi.encodePacked(
                        // ledger feed must be unique
                        ledgerDomainSeparator,
                        // Allows the verifier to detect up until which block the snapshot includes state changes
                        newBlockNumber,
                        // Bind result to previous root
                        bytes28(
                            keccak256(
                                abi.encodePacked(
                                    // ledger feed must be unique
                                    ledgerDomainSeparator,
                                    // Allows the verifier to detect up until which block the snapshot includes state
                                    // changes
                                    currentBlockNumber + 1,
                                    // Bind result to previous root
                                    bytes28(
                                        keccak256(
                                            abi.encodePacked(
                                                // ledger feed must be unique
                                                ledgerDomainSeparator,
                                                // Allows the verifier to detect up until which block the snapshot
                                                // includes state changes
                                                currentBlockNumber,
                                                // Bind result to previous root
                                                currentRootHash,
                                                // Information about the happened state change
                                                keccak256(abi.encodePacked(ChannelOpened.selector))
                                            )
                                        )
                                    ),
                                    // Information about the happened state change
                                    keccak256(abi.encodePacked(ChannelOpened.selector))
                                )
                            )
                        ),
                        // Information about the happened state change
                        keccak256(abi.encodePacked(ChannelOpened.selector))
                    )
                )
            )
        );
    }
}
