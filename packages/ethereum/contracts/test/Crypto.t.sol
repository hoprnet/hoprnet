// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { Test } from "forge-std/Test.sol";
import { AccountsFixtureTest } from "./utils/Accounts.sol";
import { CryptoUtils } from "./utils/Crypto.sol";
import { SECP2561k } from "solcrypto/SECP2561k.sol";
import { HoprCrypto } from "../src/Crypto.sol";

// Use proxy contract to have proper gas measurements for internal functions
contract CryptoProxy is HoprCrypto {
    function pointToAddressProxy(uint256 p_x, uint256 p_y) public pure returns (address) {
        return pointToAddress(p_x, p_y);
    }

    function isCurvePointInternalProxy(uint256 p_x, uint256 p_y) public pure returns (bool) {
        return isCurvePointInternal(p_x, p_y);
    }

    function isFieldElementInternalProxy(uint256 el) public pure returns (bool) {
        return isFieldElementInternal(el);
    }

    function scalarTimesBasepointProxy(uint256 scalar) public pure returns (address) {
        return scalarTimesBasepoint(scalar);
    }

    function ecAddProxy(
        uint256 p_x,
        uint256 p_y,
        uint256 q_x,
        uint256 q_y,
        uint256 a
    )
        public
        view
        returns (uint256 r_x, uint256 r_y)
    {
        return ecAdd(p_x, p_y, q_x, q_y, a);
    }

    function mapToCurveSimpleSWUProxy(uint256 u) public view returns (uint256 r_x, uint256 r_y) {
        return mapToCurveSimpleSWU(u);
    }

    function hashToScalarProxy(bytes memory message, bytes memory dst) public view returns (uint256) {
        return hashToScalar(message, dst);
    }

    function mapPointProxy(uint256 p_x, uint256 p_y) public view returns (uint256 r_x, uint256 r_y) {
        return mapPoint(p_x, p_y);
    }

    function hashToCurveProxy(bytes memory payload, bytes memory dst) public view returns (uint256 r_x, uint256 r_y) {
        return hashToCurve(payload, dst);
    }

    function vrfVerifyProxy(VRFParameters memory params, VRFPayload memory payload) public view returns (bool) {
        return vrfVerify(params, payload);
    }
}

contract Crypto is Test, AccountsFixtureTest, HoprCrypto, CryptoUtils {
    struct CurvePoint {
        uint256 x;
        uint256 y;
    }

    uint256 constant SECP256K1_BASEPOINT_X = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;
    uint256 constant SECP256K1_BASEPOINT_Y = 0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8;

    SECP2561k secp256k1;
    CryptoProxy crypto;

    function setUp() public {
        // Use proxy contract to have proper gas measurements for internal functions
        crypto = new CryptoProxy();
        secp256k1 = new SECP2561k();
    }

    function testPointToAddress() public {
        address converted = crypto.pointToAddressProxy(
            0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75,
            0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5
        );

        assertEq(converted, accountA.accountAddr);
    }

    function testIsCurvePoint() public {
        assertTrue(
            crypto.isCurvePointInternalProxy(
                0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75,
                0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5
            )
        );
    }

    function testRevert_NoCurvePoint() public {
        assertFalse(
            crypto.isCurvePointInternalProxy(
                0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5,
                0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75
            )
        );
    }

    function testScalarTimeBasepoint() public {
        assertEq(crypto.scalarTimesBasepointProxy(accountA.privateKey), accountA.accountAddr);
    }

    function testEcAddPointAddition() public {
        CurvePoint memory p = CurvePoint(
            0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75,
            0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5
        );
        CurvePoint memory q = CurvePoint(
            0xba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0,
            0xd67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4
        );
        CurvePoint memory r = CurvePoint(
            0x9d9031e97dd78ff8c15aa86939de9b1e791066a0224e331bc962a2099a7b1f04,
            0x64b8bbafe1535f2301c72c2cb3535b172da30b02686ab0393d348614f157fbdb
        );

        CurvePoint memory p_q = CurvePoint(
            0x551c7c46a964dec7edd8a5cedc557ebce43cc3f70ff481bdcfbd4e86d435c2ba,
            0x16883c2c7e2527800aa21a8420f8af48eafb2594d00e0f7e9e7d11a938b9a168
        );
        CurvePoint memory q_r = CurvePoint(
            0x85744a09c2839969dd8aa41b3577e9ffa28bee884165b880ca4050c0c3a0083e,
            0xbf55c694c111642da3ac0017e44ed12b93798eb9cbd3b14b4db1227c85e58f6d
        );

        (uint256 maybe_p_q_x, uint256 maybe_p_q_y) = crypto.ecAddProxy(p.x, p.y, q.x, q.y, 0);
        assertEq(p_q.x, maybe_p_q_x);
        assertEq(p_q.y, maybe_p_q_y);

        (uint256 maybe_q_r_x, uint256 maybe_q_r_y) = crypto.ecAddProxy(q.x, q.y, r.x, r.y, 0);
        assertEq(q_r.x, maybe_q_r_x);
        assertEq(q_r.y, maybe_q_r_y);
    }

    function testEcAddPointDouble() public {
        CurvePoint memory p = CurvePoint(
            0x9d9031e97dd78ff8c15aa86939de9b1e791066a0224e331bc962a2099a7b1f04,
            0x64b8bbafe1535f2301c72c2cb3535b172da30b02686ab0393d348614f157fbdb
        );

        CurvePoint memory p_double = CurvePoint(
            0x3d8f348848814bc251670aa3fe6301dfb7fb9f131212644cec8b666f883f1709,
            0x630684d783172d70adb684b61aed0856efe0f10982b91d57a0abe3dd08d09d32
        );

        (uint256 maybe_p_double_x, uint256 maybe_p_double_y) = crypto.ecAddProxy(p.x, p.y, p.x, p.y, 0);

        assertEq(maybe_p_double_x, p_double.x);
        assertEq(maybe_p_double_y, p_double.y);
    }

    function testEcAddFuzzy(uint256 u_0, uint256 u_1) public {
        vm.assume(crypto.isFieldElementInternalProxy(u_0));
        vm.assume(crypto.isFieldElementInternalProxy(u_1));

        (uint256 mapped_p_x, uint256 mapped_p_y) = crypto.mapToCurveSimpleSWUProxy(u_0);
        (uint256 p_x, uint256 p_y) = crypto.mapPointProxy(mapped_p_x, mapped_p_y);

        (uint256 mapped_q_x, uint256 mapped_q_y) = crypto.mapToCurveSimpleSWUProxy(u_1);
        (uint256 q_x, uint256 q_y) = crypto.mapPointProxy(mapped_q_x, mapped_q_y);

        // Q != -P
        vm.assume(p_x == q_x || p_y != q_y);

        (uint256 r_x, uint256 r_y) = crypto.ecAddProxy(p_x, p_y, q_x, q_y, 0);

        // point addition (two different points)
        assertTrue(crypto.isCurvePointInternalProxy(r_x, r_y));

        // point doubling (same point)
        (uint256 s_x, uint256 s_y) = crypto.ecAddProxy(p_x, p_y, q_x, q_y, 0);
        assertTrue(crypto.isCurvePointInternalProxy(s_x, s_y));
    }

    function testRevert_EcAddEdgeCase(uint256 u_0) public {
        u_0 = bound(u_0, 1, HoprCrypto.SECP256K1_BASE_FIELD_ORDER - 1);

        (uint256 mapped_p_x, uint256 mapped_p_y) = crypto.mapToCurveSimpleSWUProxy(u_0);
        (uint256 p_x, uint256 p_y) = crypto.mapPointProxy(mapped_p_x, mapped_p_y);

        // Test P - P
        vm.expectRevert();
        crypto.ecAddProxy(p_x, p_y, p_x, HoprCrypto.SECP256K1_BASE_FIELD_ORDER - p_y, 0);
    }

    function testSWUMap() public {
        // Test vector taken from
        // https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#appendix-J.8.1

        uint256[10] memory u = [
            0x6b0f9910dd2ba71c78f2ee9f04d73b5f4c5f7fc773a701abea1e573cab002fb3,
            0x1ae6c212e08fe1a5937f6202f929a2cc8ef4ee5b9782db68b0d5799fd8f09e16,
            0x128aab5d3679a1f7601e3bdf94ced1f43e491f544767e18a4873f397b08a2b61,
            0x5897b65da3b595a813d0fdcc75c895dc531be76a03518b044daaa0f2e4689e00,
            0xea67a7c02f2cd5d8b87715c169d055a22520f74daeb080e6180958380e2f98b9,
            0x7434d0d1a500d38380d1f9615c021857ac8d546925f5f2355319d823a478da18,
            0xeda89a5024fac0a8207a87e8cc4e85aa3bce10745d501a30deb87341b05bcdf5,
            0xdfe78cd116818fc2c16f3837fedbe2639fab012c407eac9dfe9245bf650ac51d,
            0x8d862e7e7e23d7843fe16d811d46d7e6480127a6b78838c277bca17df6900e9f,
            0x68071d2530f040f081ba818d3c7188a94c900586761e9115efa47ae9bd847938
        ];

        CurvePoint[10] memory points = [
            CurvePoint(
                0x74519ef88b32b425a095e4ebcc84d81b64e9e2c2675340a720bb1a1857b99f1e,
                0xc174fa322ab7c192e11748beed45b508e9fdb1ce046dee9c2cd3a2a86b410936
            ),
            CurvePoint(
                0x44548adb1b399263ded3510554d28b4bead34b8cf9a37b4bd0bd2ba4db87ae63,
                0x96eb8e2faf05e368efe5957c6167001760233e6dd2487516b46ae725c4cce0c6
            ),
            CurvePoint(
                0x07dd9432d426845fb19857d1b3a91722436604ccbbbadad8523b8fc38a5322d7,
                0x604588ef5138cffe3277bbd590b8550bcbe0e523bbaf1bed4014a467122eb33f
            ),
            CurvePoint(
                0xe9ef9794d15d4e77dde751e06c182782046b8dac05f8491eb88764fc65321f78,
                0xcb07ce53670d5314bf236ee2c871455c562dd76314aa41f012919fe8e7f717b3
            ),
            CurvePoint(
                0x576d43ab0260275adf11af990d130a5752704f79478628761720808862544b5d,
                0x643c4a7fb68ae6cff55edd66b809087434bbaff0c07f3f9ec4d49bb3c16623c3
            ),
            CurvePoint(
                0xf89d6d261a5e00fe5cf45e827b507643e67c2a947a20fd9ad71039f8b0e29ff8,
                0xb33855e0cc34a9176ead91c6c3acb1aacb1ce936d563bc1cee1dcffc806caf57
            ),
            CurvePoint(
                0x9c91513ccfe9520c9c645588dff5f9b4e92eaf6ad4ab6f1cd720d192eb58247a,
                0xc7371dcd0134412f221e386f8d68f49e7fa36f9037676e163d4a063fbf8a1fb8
            ),
            CurvePoint(
                0x10fee3284d7be6bd5912503b972fc52bf4761f47141a0015f1c6ae36848d869b,
                0x0b163d9b4bf21887364332be3eff3c870fa053cf508732900fc69a6eb0e1b672
            ),
            CurvePoint(
                0xb32b0ab55977b936f1e93fdc68cec775e13245e161dbfe556bbb1f72799b4181,
                0x2f5317098360b722f132d7156a94822641b615c91f8663be69169870a12af9e8
            ),
            CurvePoint(
                0x148f98780f19388b9fa93e7dc567b5a673e5fca7079cd9cdafd71982ec4c5e12,
                0x3989645d83a433bc0c001f3dac29af861f33a6fd1e04f4b36873f5bff497298a
            )
        ];

        for (uint256 i = 0; i < u.length; i++) {
            (uint256 mapped_x, uint256 mapped_y) = crypto.mapToCurveSimpleSWUProxy(u[i]);
            (uint256 p_x, uint256 p_y) = crypto.mapPointProxy(mapped_x, mapped_y);
            assertEq(p_x, points[i].x);
            assertEq(p_y, points[i].y);
        }
    }

    function testRevert_expandMessageLongDST(bytes memory message) public {
        string memory superLongDST =
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nam leo sem, consectetur facilisis nibh eget, feugiat ultrices ipsum. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Duis vel elit tempor, laoreet mauris.";

        vm.expectRevert();
        expand_message_xmd_keccak256(message, abi.encodePacked(superLongDST));

        vm.expectRevert();
        expand_message_xmd_keccak256_single(message, abi.encodePacked(superLongDST));
    }

    function testHashToCurve() public {
        bytes memory dst = "QUUX-V01-CS02-with-secp256k1_XMD:Keccak256_SSWU_RO_";

        // test strings taken from https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#appendix-J.8.1
        bytes[5] memory testStrings = [
            bytes(""),
            bytes("abc"),
            bytes("abcdef0123456789"),
            bytes(
                "q128_qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq"
            ),
            bytes(
                "a512_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            )
        ];

        // Generated with Rust implementation
        // sample Rust code:
        // ```rust
        // use elliptic_curve::hash2curve::{ExpandMsgXmd, GroupDigest};
        // use k256::Secp256k1;
        //
        // let hash = Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(msg);
        // ```
        CurvePoint[5] memory points = [
            CurvePoint(
                0xa8d5be3d37133158c01970d186839bc7405fb26c0c8c9687c5a0783f3e23db6d,
                0xfa9b1660a78cfe5a60cdb6355fde4d4108bcfb58cc2b97b655e629c0604849bf
            ),
            CurvePoint(
                0xd7e69a5226454f72a551c0799460d068fd1ffff6445146fb3beb9a842d5affbd,
                0x6de9462bd1fe58a603945b88927724f20d2ac0671223195f21d41609ce4c1265
            ),
            CurvePoint(
                0xefe470da01abba8406af26987fd94d7e8cfb917c75e4d4a005c4da40be588035,
                0x4883332ba13cc90cdcdb4db0eea6e0426360c8a218e82c8823a4609e564e7ee9
            ),
            CurvePoint(
                0xc1f6f9a3d1f7268162d3c7f4f2e2853dfd7cc302ad70f1f449e6c3e3668b97ac,
                0x67040bae5790efe2aa6524b413e9b7949540b4f2839cc43adbb875ad5da2b4b0
            ),
            CurvePoint(
                0x7140c0230d96c79cd4c70de14c69eb89c35fa53b3b71b9580d2b2fd872af6d7c,
                0xdd315906c024f1609504fe1dbf432d019e645fb9f3ce23e94ddcdd0a225acafc
            )
        ];

        for (uint256 i = 0; i < testStrings.length; i++) {
            (uint256 p_x, uint256 p_y) = crypto.hashToCurveProxy(testStrings[i], dst);
            CurvePoint memory should = points[i];

            assertEq(p_x, should.x);
            assertEq(p_y, should.y);
        }
    }

    function testFuzzyHashToCurve(bytes memory vrfMessage) public {
        string memory dst = "some DST tag";

        (uint256 p_x, uint256 p_y) = crypto.hashToCurveProxy(vrfMessage, abi.encodePacked(dst));

        assertTrue(crypto.isCurvePointInternalProxy(p_x, p_y));
    }

    function testVRFVerify() public {
        // sample Rust code:
        // ```rust
        // use elliptic_curve::{hash2curve::{ExpandMsgXmd, GroupDigest}, ScalarPrimitive};
        // use k256::{AffinePoint, Scalar, Secp256k1};
        //
        // let b = Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[&chain_addr.to_bytes(), msg], &[dst])?;
        // let a: Scalar = ScalarPrimitive::<Secp256k1>::from_slice(&secret)?.into();
        // if a.is_zero().into() {
        //     return Err(crate::errors::CryptoError::InvalidSecretScalar);
        // }
        //
        // let v = b * a;
        // let r = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        //     &[
        //         &a.to_bytes(),
        //         &v.to_affine().to_encoded_point(false).as_bytes()[1..],
        //         &random_bytes::<64>(),
        //     ],
        //     &[dst],
        // )?;
        //
        // let r_v = b * r;
        //
        // let h = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        //     &[
        //         &chain_addr.to_bytes(),
        //         &v.to_affine().to_encoded_point(false).as_bytes()[1..],
        //         &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
        //         msg,
        //     ],
        //     &[dst],
        // )?;
        // let s = r + h * a;
        // ```
        // precomputed values with Rust implementation
        CurvePoint memory V = CurvePoint(
            0x7e4d7332351201f79215328221cf4baeb9365cdba1255cedfe1cc635b4780a0f,
            0xd9fe81cfdd0537cd5f73120ab2f97caa93bddf4ce8922b99c6649ecb2b15ddd7
        );
        bytes32 h = 0x0f6be5484fce28779f0ff72d4f68c619b3f96f1af895c7c6ec4aede81534cee8;
        bytes32 s = 0xfdab2e0a3e314d1f2b0681e7c62e65345563907ab09751ec6292e039345f959f;

        CurvePoint memory h_V = CurvePoint(
            0xa02218468ac06b30714a92a92dacca5a28a8a035efffecf03dae372646ad5489,
            0xa2dbfa875633e485399a8b0a933b2480129c90725ecc6bbdb906d27a4231a744
        );
        CurvePoint memory s_B = CurvePoint(
            0xb2eff54b867ba38bd60631df94a4dcba261bdd88f6fa1d38dee11053ec0ec9c4,
            0xfa1f1e9cacd0b9c895e2d00e4fbedc791e2b176b261ba13ec73aaeb3e5d430f6
        );

        address signer = 0x0c6146f8e9A92174A309bd0d7f000148fD6e2588;

        HoprCrypto.VRFParameters memory params =
            HoprCrypto.VRFParameters(V.x, V.y, uint256(s), uint256(h), s_B.x, s_B.y, h_V.x, h_V.y);

        HoprCrypto.VRFPayload memory payload = HoprCrypto.VRFPayload(
            0xf13233ff60e1f618525dac5f7d117bef0bad0eb0b0afb2459f9cbc57a3a987ba, signer, "some DST tag"
        );

        assertTrue(crypto.vrfVerifyProxy(params, payload));
    }

    function testFuzzVRFVerify(uint256 privKey, bytes32 vrfMessage) public {
        string memory dst = "some DST tag";

        privKey = bound(privKey, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);

        HoprCrypto.VRFPayload memory payload;

        address chain_addr = crypto.scalarTimesBasepointProxy(privKey);
        payload.message = vrfMessage;
        payload.signer = chain_addr;
        payload.dst = abi.encodePacked(dst);

        HoprCrypto.VRFParameters memory params =
            CryptoUtils.getVRFParameters(privKey, abi.encodePacked(dst), vrfMessage);

        assertTrue(crypto.vrfVerifyProxy(params, payload));
    }

    function testRevert_vrfVerifyInvalidFieldElement(
        uint256 sTweaked,
        uint256 hTweaked,
        uint256 privKey,
        bytes32 vrfMessage
    )
        public
    {
        sTweaked = bound(sTweaked, HoprCrypto.SECP256K1_BASE_FIELD_ORDER, type(uint256).max);
        hTweaked = bound(hTweaked, HoprCrypto.SECP256K1_BASE_FIELD_ORDER, type(uint256).max);

        string memory dst = "some DST tag";

        privKey = bound(privKey, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);

        HoprCrypto.VRFPayload memory payload;

        address chain_addr = crypto.scalarTimesBasepointProxy(privKey);
        payload.message = vrfMessage;
        payload.signer = chain_addr;
        payload.dst = abi.encodePacked(dst);

        HoprCrypto.VRFParameters memory params =
            CryptoUtils.getVRFParameters(privKey, abi.encodePacked(dst), vrfMessage);

        uint256 tmp = params.s;
        params.s = sTweaked;

        vm.expectRevert(HoprCrypto.InvalidFieldElement.selector);
        crypto.vrfVerifyProxy(params, payload);

        // reset value
        params.s = tmp;

        params.h = hTweaked;

        vm.expectRevert(HoprCrypto.InvalidFieldElement.selector);
        crypto.vrfVerifyProxy(params, payload);
    }

    function testRevert_vrfVerifyInvalidCurvePoint(
        uint256 pxTweaked,
        uint256 pyTweaked,
        uint256 privKey,
        bytes32 vrfMessage
    )
        public
    {
        pxTweaked = bound(pxTweaked, 0, HoprCrypto.SECP256K1_BASE_FIELD_ORDER - 1);
        pyTweaked = bound(pyTweaked, 0, HoprCrypto.SECP256K1_BASE_FIELD_ORDER - 1);

        // there should be plenty of these points
        vm.assume(!crypto.isCurvePointInternalProxy(pxTweaked, pyTweaked));

        string memory dst = "some DST tag";

        privKey = bound(privKey, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);

        HoprCrypto.VRFPayload memory payload;

        address chain_addr = crypto.scalarTimesBasepointProxy(privKey);
        payload.message = vrfMessage;
        payload.signer = chain_addr;
        payload.dst = abi.encodePacked(dst);

        HoprCrypto.VRFParameters memory params =
            CryptoUtils.getVRFParameters(privKey, abi.encodePacked(dst), vrfMessage);

        params.vx = pxTweaked;
        params.vy = pyTweaked;

        vm.expectRevert(HoprCrypto.InvalidCurvePoint.selector);
        crypto.vrfVerifyProxy(params, payload);
    }

    function testRevert_vrfVerifyInvalidPointWitness(
        uint256 sTweaked,
        uint256 hTweaked,
        uint256 privKey,
        bytes32 vrfMessage
    )
        public
    {
        sTweaked = bound(sTweaked, 1, HoprCrypto.SECP256K1_BASE_FIELD_ORDER - 1);
        hTweaked = bound(hTweaked, 1, HoprCrypto.SECP256K1_BASE_FIELD_ORDER - 1);

        string memory dst = "some DST tag";

        privKey = bound(privKey, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);

        HoprCrypto.VRFPayload memory payload;

        address chain_addr = crypto.scalarTimesBasepointProxy(privKey);
        payload.message = vrfMessage;
        payload.signer = chain_addr;
        payload.dst = abi.encodePacked(dst);

        HoprCrypto.VRFParameters memory params =
            CryptoUtils.getVRFParameters(privKey, abi.encodePacked(dst), vrfMessage);

        vm.assume(sTweaked != params.s);
        vm.assume(hTweaked != params.h);

        // use different scalar
        (uint256 rx, uint256 ry) = ecmul(SECP256K1_BASEPOINT_X, SECP256K1_BASEPOINT_Y, sTweaked);

        uint256 tmpX = params.hVx;
        uint256 tmpY = params.hVy;

        params.hVx = rx;
        params.hVy = ry;

        vm.expectRevert(HoprCrypto.InvalidPointWitness.selector);
        crypto.vrfVerifyProxy(params, payload);

        params.hVx = tmpX;
        params.hVy = tmpY;

        (rx, ry) = ecmul(SECP256K1_BASEPOINT_X, SECP256K1_BASEPOINT_Y, hTweaked);

        params.sBx = rx;
        params.sBy = ry;

        vm.expectRevert(HoprCrypto.InvalidPointWitness.selector);
        crypto.vrfVerifyProxy(params, payload);
    }

    function test_vrfVerifyFail(uint256 hTweaked, uint256 privKey, bytes32 vrfMessage) public {
        hTweaked = bound(hTweaked, 1, HoprCrypto.SECP256K1_BASE_FIELD_ORDER - 1);

        string memory dst = "some DST tag";

        privKey = bound(privKey, 1, HoprCrypto.SECP256K1_FIELD_ORDER - 1);

        HoprCrypto.VRFPayload memory payload;

        address chain_addr = crypto.scalarTimesBasepointProxy(privKey);
        payload.message = vrfMessage;
        payload.signer = chain_addr;
        payload.dst = abi.encodePacked(dst);

        HoprCrypto.VRFParameters memory params =
            CryptoUtils.getVRFParameters(privKey, abi.encodePacked(dst), vrfMessage);

        vm.assume(hTweaked != params.h);

        (uint256 rx, uint256 ry) = ecmul(params.vx, params.vy, hTweaked);

        params.h = hTweaked;
        params.hVx = rx;
        params.hVy = ry;

        assertFalse(crypto.vrfVerifyProxy(params, payload));
    }
}
