In this folder we have [copied](./ERC777.sol) openzeppelin's ERC777 contract from commit [3b4c951838ddf82f4d2b44ae7c967f0573039f28](https://github.com/OpenZeppelin/openzeppelin-contracts/commit/3b4c951838ddf82f4d2b44ae7c967f0573039f28) as it fixes a [potential reentrancy issue](https://github.com/OpenZeppelin/openzeppelin-contracts/pull/2483).

At the time of making, an official release for the reentrancy issue is not available, at the same time, since our `@openzeppelin/contracts` dependancy is significantly older than the latest `v3.3.0` it may require various other changes in the source code and would require a change in the audit scope.

We _have_ made some changes to the [copied](./ERC777.sol) ERC777 contract so to point to the right paths.

```diff
- import "../../utils/Context.sol";
- import "./IERC777.sol";
- import "./IERC777Recipient.sol";
- import "./IERC777Sender.sol";
- import "../../token/ERC20/IERC20.sol";
- import "../../math/SafeMath.sol";
- import "../../utils/Address.sol";
- import "../../introspection/IERC1820Registry.sol";
+ import "@openzeppelin/contracts/GSN/Context.sol";
+ import "@openzeppelin/contracts/token/ERC777/IERC777.sol";
+ import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
+ import "@openzeppelin/contracts/token/ERC777/IERC777Sender.sol";
+ import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
+ import "@openzeppelin/contracts/math/SafeMath.sol";
+ import "@openzeppelin/contracts/utils/Address.sol";
+ import "@openzeppelin/contracts/introspection/IERC1820Registry.sol";
```

We need to consider that this change means that we have now copied a smart contract that was originally pointing to different files with different source code.
We have compiled a list of comparisons ([v3.0.1](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v3.0.1) to [3b4c951838ddf82f4d2b44ae7c967f0573039f28](https://github.com/OpenZeppelin/openzeppelin-contracts/commit/3b4c951838ddf82f4d2b44ae7c967f0573039f28)) to evaluate the [differences](https://github.com/OpenZeppelin/openzeppelin-contracts/compare/v3.0.1...3b4c951838ddf82f4d2b44ae7c967f0573039f28) between each files.

- GSN/Context.sol [diff](https://github.com/OpenZeppelin/openzeppelin-contracts/compare/v3.0.1...3b4c951838ddf82f4d2b44ae7c967f0573039f28#diff-1ec769e311166f565ff732cce2b1ea77ffcad78c2c6fae3fe2487ac578f6cc67): moved file into utils & updated comments
- token/ERC777/IERC777.sol [diff](https://github.com/OpenZeppelin/openzeppelin-contracts/compare/v3.0.1...3b4c951838ddf82f4d2b44ae7c967f0573039f28#diff-0a1db262e475195b998705b0b403e96e6f27880caa538c64fc621f821239b4bf): pragma changed
- token/ERC777/IERC777Recipient.sol [diff](https://github.com/OpenZeppelin/openzeppelin-contracts/compare/v3.0.1...3b4c951838ddf82f4d2b44ae7c967f0573039f28#diff-ad4b17d569eefb57bab74f0bb4f2384e06b43f5a88fffe3727112fd29896e04f): pragma changed
- token/ERC777/IERC777Sender.sol [diff](https://github.com/OpenZeppelin/openzeppelin-contracts/compare/v3.0.1...3b4c951838ddf82f4d2b44ae7c967f0573039f28#diff-e6308242e8c1f92d6c9c2144a149b8470ea69c1158024c8d91d4acd7b8c1e01c): pragma changed
- token/ERC20/IERC20.sol [diff](https://github.com/OpenZeppelin/openzeppelin-contracts/compare/v3.0.1...3b4c951838ddf82f4d2b44ae7c967f0573039f28#diff-4d0f542e38c77e0be35b1bf2be90a71faa238ec029684b6fd0a0fb0b01ee2917): pragma changed
- math/SafeMath.sol [diff](https://github.com/OpenZeppelin/openzeppelin-contracts/compare/v3.0.1...3b4c951838ddf82f4d2b44ae7c967f0573039f28#diff-2756602dafa80f54b5620a9c18731eb09dc0b9dcd849bb30f47b7474956cb1e1): various changes
- utils/Address.sol [diff](https://github.com/OpenZeppelin/openzeppelin-contracts/compare/v3.0.1...3b4c951838ddf82f4d2b44ae7c967f0573039f28#diff-3723538f11e9073477613a08f0f092e286fcab832b33b66b4a3d8f153cbe4c34): various changes
- introspection/IERC1820Registry.sol [diff](https://github.com/OpenZeppelin/openzeppelin-contracts/compare/v3.0.1...3b4c951838ddf82f4d2b44ae7c967f0573039f28#diff-276e7e904e538a49c660974feafa69634d150cdccd9d7871808cccd288d72456): changed pragma & cosmetic change
