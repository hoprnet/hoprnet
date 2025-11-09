// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8;

import { HoprCrypto } from "../../src/Crypto.sol";
import { HoprChannels } from "../../src/Channels.sol";
import { SECP2561k } from "solcrypto/SECP2561k.sol";
import { Test } from "forge-std/Test.sol";

abstract contract CryptoUtils is Test, HoprCrypto, SECP2561k {
    uint256 constant SECP256K1_HALF_FIELD_ORDER = 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0;

    struct RedeemTicketArgBuilder {
        uint256 privKeyA;
        uint256 privKeyB;
        bytes32 dst;
        address src;
        address dest;
        uint256 amount;
        uint256 maxTicketIndex;
        uint256 indexOffset;
        uint256 epoch;
        uint256 winProb;
        uint256 porSecret;
    }

    function _getChannelId(address source, address destination) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(source, destination));
    }

    function getRedeemableTicket(RedeemTicketArgBuilder memory args)
        internal
        view
        returns (HoprChannels.RedeemableTicket memory redeemable, VRFParameters memory vrf)
    {
        bytes32 channelId = _getChannelId(args.src, args.dest);

        HoprChannels.TicketData memory ticketData = HoprChannels.TicketData(
            channelId,
            HoprChannels.Balance.wrap(uint96(args.amount)),
            HoprChannels.TicketIndex.wrap(uint48(args.maxTicketIndex)),
            HoprChannels.TicketIndexOffset.wrap(uint32(args.indexOffset)),
            HoprChannels.ChannelEpoch.wrap(uint24(args.epoch)),
            HoprChannels.WinProb.wrap(uint56(args.winProb))
        );

        address challenge = HoprCrypto.scalarTimesBasepoint(args.porSecret);

        uint256 secondPart = (args.amount << 160) | (args.maxTicketIndex << 112) | (args.indexOffset << 80)
            | (args.epoch << 56) | args.winProb;

        // Deviates from EIP712 due to computed property and non-standard struct property encoding
        bytes32 hashStruct = keccak256(
            abi.encode(
                HoprChannels.redeemTicket.selector, keccak256(abi.encodePacked(channelId, secondPart, challenge))
            )
        );

        bytes32 ticketHash = keccak256(abi.encodePacked(bytes1(0x19), bytes1(0x01), args.dst, hashStruct));

        CompactSignature memory sig;

        {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(args.privKeyA, ticketHash);

            sig = toCompactSignature(v, r, s);
        }

        redeemable = HoprChannels.RedeemableTicket(ticketData, sig, args.porSecret);

        vrf = getVRFParameters(args.privKeyB, abi.encodePacked(args.dst), ticketHash);
    }

    function toCompactSignature(
        uint8 v,
        bytes32 r,
        bytes32 s
    )
        internal
        pure
        returns (HoprCrypto.CompactSignature memory sig)
    {
        if (uint256(s) >= SECP256K1_HALF_FIELD_ORDER) {
            s = bytes32(HoprCrypto.SECP256K1_FIELD_ORDER - uint256(s));
        }
        sig.r = r;
        sig.vs = bytes32(uint256(v - 27) << 255) | s;
    }

    function decompressSignature(
        bytes32 r,
        bytes32 vs
    )
        internal
        pure
        returns (uint8 v_out, bytes32 r_out, bytes32 s_out)
    {
        s_out = vs & bytes32(0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
        v_out = uint8((uint256(vs) >> 255) + 27);
        r_out = r;
    }

    function getVRFParameters(
        uint256 privKey,
        bytes memory dst,
        bytes32 vrfMessage
    )
        internal
        view
        returns (HoprCrypto.VRFParameters memory params)
    {
        HoprCrypto.VRFPayload memory payload;

        // stack height optimization, doesn't compile otherwise
        {
            address chain_addr = HoprCrypto.scalarTimesBasepoint(privKey);
            payload.message = vrfMessage;
            payload.signer = chain_addr;
            payload.dst = abi.encodePacked(dst);
        }

        // stack height optimization, doesn't compile otherwise
        {
            (uint256 bx, uint256 by) =
                HoprCrypto.hashToCurve(abi.encodePacked(payload.signer, payload.message), payload.dst);

            {
                (uint256 vx, uint256 vy) = SECP2561k.ecmul(bx, by, privKey);
                params.vx = vx;
                params.vy = vy;
            }

            uint256 r = HoprCrypto.hashToScalar(abi.encodePacked(privKey, bx, by, payload.message), payload.dst);

            (uint256 brx, uint256 bry) = SECP2561k.ecmul(bx, by, r);

            params.h = HoprCrypto.hashToScalar(
                abi.encodePacked(payload.signer, params.vx, params.vy, brx, bry, payload.message), abi.encodePacked(dst)
            );

            // s = r + h * a (mod p)
            params.s =
                addmod(r, mulmod(params.h, privKey, HoprCrypto.SECP256K1_FIELD_ORDER), HoprCrypto.SECP256K1_FIELD_ORDER);

            {
                (uint256 sBx, uint256 sBy) = SECP2561k.ecmul(bx, by, params.s);
                params.sBx = sBx;
                params.sBy = sBy;

                (uint256 h_v_x, uint256 h_v_y) = SECP2561k.ecmul(params.vx, params.vy, params.h);
                params.hVx = h_v_x;
                params.hVy = h_v_y;
            }
        }
    }
}
