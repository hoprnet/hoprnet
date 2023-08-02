// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

import "../../src/Crypto.sol";
import "../../src/Channels.sol";
import "solcrypto/SECP2561k.sol";
import "forge-std/Test.sol";

abstract contract CryptoUtils is Test, HoprCrypto, SECP2561k {
    uint256 constant SECP256K1_HALF_FIELD_ORDER = 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0;

    struct RedeemTicketArgBuilder {
        uint256 privKeyA;
        uint256 privKeyB;
        bytes DST;
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

    function getRedeemableTicket(RedeemTicketArgBuilder memory args, bytes32 dstHash)
        internal
        view
        returns (HoprChannels.RedeemableTicket memory redeemable, VRF_Parameters memory vrf)
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

        bytes32 ticketHash;
        {
            bytes32 pre_ticketHash = keccak256(
                abi.encode(
                    HoprChannels.redeemTicket.selector, keccak256(abi.encode(ticketData, vm.addr(args.porSecret)))
                )
            );
            ticketHash = keccak256(abi.encode(bytes1(0x19), bytes1(0x01), dstHash, pre_ticketHash));
        }

        CompactSignature memory sig;
        {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(args.privKeyA, ticketHash);

            sig = toCompactSignature(v, r, s);
        }

        redeemable = HoprChannels.RedeemableTicket(ticketData, sig, args.porSecret);

        vrf = getVRFParameters(args.privKeyB, args.DST, ticketHash);
    }

    function toCompactSignature(uint8 v, bytes32 r, bytes32 s)
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

    function decompressSignature(bytes32 r, bytes32 vs)
        internal
        pure
        returns (uint8 v_out, bytes32 r_out, bytes32 s_out)
    {
        s_out = vs & bytes32(0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
        v_out = uint8((uint256(vs) >> 255) + 27);
        r_out = r;
    }

    function getVRFParameters(uint256 privKey, bytes memory DST, bytes32 vrfMessage)
        internal
        view
        returns (HoprCrypto.VRF_Parameters memory params)
    {
        HoprCrypto.VRF_Payload memory payload;

        // stack height optimization, doesn't compile otherwise
        {
            address chain_addr = HoprCrypto.scalarTimesBasepoint(privKey);
            payload.message = vrfMessage;
            payload.signer = chain_addr;
            payload.DST = abi.encodePacked(DST);
        }

        // stack height optimization, doesn't compile otherwise
        {
            (uint256 b_x, uint256 b_y) =
                HoprCrypto.hashToCurve(abi.encodePacked(payload.signer, payload.message), payload.DST);

            {
                (uint256 v_x, uint256 v_y) = SECP2561k.ecmul(b_x, b_y, privKey);
                params.v_x = v_x;
                params.v_y = v_y;
            }

            uint256 r = HoprCrypto.hashToScalar(abi.encodePacked(privKey, b_x, b_y, payload.message), payload.DST);

            (uint256 b_r_x, uint256 b_r_y) = SECP2561k.ecmul(b_x, b_y, r);

            params.h = HoprCrypto.hashToScalar(
                abi.encodePacked(payload.signer, params.v_x, params.v_y, b_r_x, b_r_y, payload.message),
                abi.encodePacked(DST)
            );

            // s = r + h * a (mod p)
            params.s =
                addmod(r, mulmod(params.h, privKey, HoprCrypto.SECP256K1_FIELD_ORDER), HoprCrypto.SECP256K1_FIELD_ORDER);

            {
                (uint256 s_b_x, uint256 s_b_y) = SECP2561k.ecmul(b_x, b_y, params.s);
                params.sB_x = s_b_x;
                params.sB_y = s_b_y;

                (uint256 h_v_x, uint256 h_v_y) = SECP2561k.ecmul(params.v_x, params.v_y, params.h);
                params.hV_x = h_v_x;
                params.hV_y = h_v_y;
            }
        }
    }
}
