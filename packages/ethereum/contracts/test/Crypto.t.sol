// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import './utils/Accounts.sol';

import '../src/Crypto.sol';

contract Crypto is Test, AccountsFixtureTest, HoprCrypto {
  function modifierIsCurvePoint(CurvePoint memory p) isCurvePoint(p) public {}

  function setUp() public {
  }

  function testInvMod(uint256 el) public {
    el = bound(el, 2 , HoprCrypto.SECP256K1_BASE_FIELD_ORDER - 1);

    uint256 invEl = HoprCrypto.invMod(el);

    assertEq(mulmod(el, invEl, HoprCrypto.SECP256K1_BASE_FIELD_ORDER), 1);
  }

  function testPointToAddress() public {
    (bool success, bytes memory returnValue) = address(this).staticcall(abi.encodeWithSelector(HoprCrypto.pointToAddress.selector, 0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75, 0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5));

    assertTrue(success);
    assertEq(address(uint160(uint256(bytes32(returnValue)))), accountA.accountAddr);
  }

  function testIsCurvePoint() public { 
    modifierIsCurvePoint(CurvePoint(0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75, 0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5));
  }

  function testRevert_NoCurvePoint() public {
    vm.expectRevert(InvalidCurvePoint.selector);
    modifierIsCurvePoint(CurvePoint(0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5, 0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75));
  }

  function testScalarTimeBasepoint() public {
    assertEq(HoprCrypto.scalarTimesBasepoint(accountA.privateKey), accountA.accountAddr);
  }

  function testEcAdd() public {
    CurvePoint memory p = CurvePoint(0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75,0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5);
    CurvePoint memory q = CurvePoint(0xba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0,0xd67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4);
    CurvePoint memory r = CurvePoint(0x9d9031e97dd78ff8c15aa86939de9b1e791066a0224e331bc962a2099a7b1f04,0x64b8bbafe1535f2301c72c2cb3535b172da30b02686ab0393d348614f157fbdb);

    CurvePoint memory p_q = CurvePoint(0x551c7c46a964dec7edd8a5cedc557ebce43cc3f70ff481bdcfbd4e86d435c2ba,0x16883c2c7e2527800aa21a8420f8af48eafb2594d00e0f7e9e7d11a938b9a168);
    CurvePoint memory q_r = CurvePoint(0x85744a09c2839969dd8aa41b3577e9ffa28bee884165b880ca4050c0c3a0083e,0xbf55c694c111642da3ac0017e44ed12b93798eb9cbd3b14b4db1227c85e58f6d);
    
    CurvePoint memory maybe_p_q = ecAdd(p, q);
    assertEq(p_q.x, maybe_p_q.x);
    assertEq(p_q.y, maybe_p_q.y);

    CurvePoint memory maybe_q_r = ecAdd(q, r);
    assertEq(q_r.x, maybe_q_r.x);
    assertEq(q_r.y, maybe_q_r.y);
  }

  function testSqrt() public {
    uint256 x = 11;
    
    (bool success, bytes memory returnValue) = address(this).staticcall(abi.encodeWithSelector(HoprCrypto.sqrtMod.selector, x));
    uint256 result = abi.decode(returnValue, (uint256));

    assertTrue(success);
    assertEq(mulmod(result, result, SECP256K1_BASE_FIELD_ORDER), x);
    console.log(result);

    console.logBytes32(bytes32((SECP256K1_BASE_FIELD_ORDER - 3) / 4));
  }

  // function testIsSquare() public {
  //   uint256 SQUARE = 11;
  //   uint256 NON_SQUARE = SECP256K1_BASE_FIELD_ORDER - 11;

  //   assertTrue(HoprCrypto.isSquare(SQUARE));
  //   assertFalse(HoprCrypto.isSquare(NON_SQUARE));
  // }

  // function testMapPoint() public {

  // }

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
      CurvePoint(0x74519ef88b32b425a095e4ebcc84d81b64e9e2c2675340a720bb1a1857b99f1e,0xc174fa322ab7c192e11748beed45b508e9fdb1ce046dee9c2cd3a2a86b410936),
      CurvePoint(0x44548adb1b399263ded3510554d28b4bead34b8cf9a37b4bd0bd2ba4db87ae63,0x96eb8e2faf05e368efe5957c6167001760233e6dd2487516b46ae725c4cce0c6),
      CurvePoint(0x07dd9432d426845fb19857d1b3a91722436604ccbbbadad8523b8fc38a5322d7,0x604588ef5138cffe3277bbd590b8550bcbe0e523bbaf1bed4014a467122eb33f),
      CurvePoint(0xe9ef9794d15d4e77dde751e06c182782046b8dac05f8491eb88764fc65321f78,0xcb07ce53670d5314bf236ee2c871455c562dd76314aa41f012919fe8e7f717b3),
      CurvePoint(0x576d43ab0260275adf11af990d130a5752704f79478628761720808862544b5d,0x643c4a7fb68ae6cff55edd66b809087434bbaff0c07f3f9ec4d49bb3c16623c3),
      CurvePoint(0xf89d6d261a5e00fe5cf45e827b507643e67c2a947a20fd9ad71039f8b0e29ff8,0xb33855e0cc34a9176ead91c6c3acb1aacb1ce936d563bc1cee1dcffc806caf57),
      CurvePoint(0x9c91513ccfe9520c9c645588dff5f9b4e92eaf6ad4ab6f1cd720d192eb58247a,0xc7371dcd0134412f221e386f8d68f49e7fa36f9037676e163d4a063fbf8a1fb8),
      CurvePoint(0x10fee3284d7be6bd5912503b972fc52bf4761f47141a0015f1c6ae36848d869b,0x0b163d9b4bf21887364332be3eff3c870fa053cf508732900fc69a6eb0e1b672),
      CurvePoint(0xb32b0ab55977b936f1e93fdc68cec775e13245e161dbfe556bbb1f72799b4181,0x2f5317098360b722f132d7156a94822641b615c91f8663be69169870a12af9e8),
      CurvePoint(0x148f98780f19388b9fa93e7dc567b5a673e5fca7079cd9cdafd71982ec4c5e12,0x3989645d83a433bc0c001f3dac29af861f33a6fd1e04f4b36873f5bff497298a)  
    ];

    for (uint i = 0; i < u.length; i++) {
      CurvePoint memory p = mapPoint(map_to_curve_simple_swu(u[i]));
      assertEq(p.x, points[i].x);
      assertEq(p.y, points[i].y);
    }
  }

  function testExpandMsgXmd() public {
    bytes memory DST = "QUUX-V01-CS02-with-expander-SHA256-128";

    bytes[5] memory testStrings = [
      bytes(""),
      bytes("abc"),
      bytes("abcdef0123456789"),
      bytes("q128_qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq"),
      bytes("a512_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    ];

    uint256[15] memory hashValue = [
      0x3432c9c0a960eeed516f6b01a3b605dc1eb3384cc5ca30dc6878563573ff7f0d,
      0x37bdc07c1644d3686bb9571f67f110196b9c542a61b2674e3a8e0ac89cf46dd2,
      0xb5bda8d8841e493fc9606f8cf16aad07e3d3ca015f8086a519377449b10ae80a,
      0x15873f85fe27c01a13ce8a6d546e912e21a44f758cb4ceafdf7ce15f767106fa,
      0x3ed2c1f928798606343e6aa60ca9d4d4d035cfcaccb3133f712df12d882bd800,
      0x1a7e45c260f68f22d3f8a2e0b0d0825dbf0fc42386df81413d1435e0cce8fee5,
      0xd19a5f98a7232b23563eb994f8e51d984fb4229b9eaf2e02854ac029c4faa0bf,
      0x5f3ad1cc12e4e85509d86beaeae050a248b95dbe8efbcd3bf2d1141d59d0650d,
      0xfd5149675a4057dd5a51843c1fdcffe3e1ef177c2f0068fc5999f1b99adbe140,
      0x8c5fc5fd3358f7000fd3738d710a718bf5546020c65182d22edb73d012e734b7,
      0x47dfb3c74fefeee6c7125245923af62757c45223e07d638b7aeb920743359fa3,
      0x85485929b734d02783a664ffa98008bf77831606e64f285e49bf1b499a8ed568,
      0xeec447c0cd64d426e3b9e71e2b7049330a20a5e1cddd0cb480fa4a326d5f0109,
      0x8ba30055c478728248607000dd1892a9a38be8a2eb96d97458c7e9e667f58a34,
      0xca03f7c3a51636cb786cb994fdc102ed8cf75e7473a4558bb0c3f551f027e3d8
    ];

    for (uint256 i = 0; i < 5; i++) {
      (bytes32 u_0, bytes32 u_1, bytes32 u_2) = expand_message_xmd(testStrings[i], DST);
      assertEq(u_0, bytes32(hashValue[3*i]));
      assertEq(u_1, bytes32(hashValue[3*i + 1])); 
      assertEq(u_2, bytes32(hashValue[3*i + 2])); 
    }
  }

  function testHashToCurve() public {
    bytes memory DST = "QUUX-V01-CS02-with-secp256k1_XMD:SHA-256_SSWU_RO_";

    // test vector taken from https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#appendix-J.8.1
    bytes[5] memory testStrings = [
      bytes(""),
      bytes("abc"),
      bytes("abcdef0123456789"),
      bytes("q128_qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq"),
      bytes("a512_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    ];

    CurvePoint[5] memory points = [
      CurvePoint(0xc1cae290e291aee617ebaef1be6d73861479c48b841eaba9b7b5852ddfeb1346,0x64fa678e07ae116126f08b022a94af6de15985c996c3a91b64c406a960e51067),
      CurvePoint(0x3377e01eab42db296b512293120c6cee72b6ecf9f9205760bd9ff11fb3cb2c4b,0x7f95890f33efebd1044d382a01b1bee0900fb6116f94688d487c6c7b9c8371f6),
      CurvePoint(0xbac54083f293f1fe08e4a70137260aa90783a5cb84d3f35848b324d0674b0e3a,0x4436476085d4c3c4508b60fcf4389c40176adce756b398bdee27bca19758d828),
      CurvePoint(0xe2167bc785333a37aa562f021f1e881defb853839babf52a7f72b102e41890e9,0xf2401dd95cc35867ffed4f367cd564763719fbc6a53e969fb8496a1e6685d873),
      CurvePoint(0xe3c8d35aaaf0b9b647e88a0a0a7ee5d5bed5ad38238152e4e6fd8c1f8cb7c998,0x8446eeb6181bf12f56a9d24e262221cc2f0c4725c7e3803024b5888ee5823aa6)
    ];

    for (uint256 i = 0; i < 5; i++) {
      CurvePoint memory p = hashToCurve(testStrings[i], DST);
      CurvePoint memory should = points[i];

      assertEq(p.x, should.x);
      assertEq(p.y, should.y);
    }
  }
}