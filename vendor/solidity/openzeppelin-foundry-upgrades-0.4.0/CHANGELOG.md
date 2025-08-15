# Changelog

## 0.4.0 (2025-01-27)

- Remove dependency on `solidity-stringutils`. ([#91](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/91))

### Breaking changes
- Requires `forge-std` version v1.9.5 or higher.

## 0.3.8 (2025-01-24)

- Fix error conditions when warnings occur in validation output. ([#94](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/94))

## 0.3.7 (2025-01-13)

- Update documentation links. ([#88](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/88))
- Set Foundry as client origin for Defender deployments. ([#90](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/90))

## 0.3.6 (2024-09-24)

- Add `referenceBuildInfoDir` and `exclude` options. ([#74](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/74))

## 0.3.5 (2024-09-20)

### Potentially breaking changes
- Adds a check to ensure `initialOwner` for `deployTransparentProxy` is not a ProxyAdmin contract. ([#76](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/76))

## 0.3.4 (2024-09-16)

- Defender: Add `metadata` option. ([#75](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/75))

## 0.3.3 (2024-08-26)

- Defender: Fix error in `proposeUpgrade` when project path has a space. ([#71](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/71))

## 0.3.2 (2024-08-14)

- Fix simulation failure due to revert when upgrading deployments using OpenZeppelin Contracts v4. ([#65](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/65))

## 0.3.1 (2024-05-21)

- Fix upgrade interface version detection in `upgradeProxy` function. ([#53](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/53))

## 0.3.0 (2024-05-14)

- Adds library variations to support `forge coverage` or upgrade existing deployments using OpenZeppelin Contracts v4. ([#50](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/50))

### Breaking changes
- Removed the `CHEATCODE_ADDRESS` internal constant from `Upgrades.sol`.

## 0.2.3 (2024-05-02)

- Defender: Add `txOverrides` option. ([#49](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/49))

## 0.2.2 (2024-04-17)

- Defender: Fix handling of license types for block explorer verification, support `licenseType` and `skipLicenseType` options. ([#43](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/43))

## 0.2.1 (2024-03-20)

- Throw helpful error message if AST not found in contract artifacts. ([#28](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/28))

## 0.2.0 (2024-03-20)

- Update forge-std to v1.8.0, restrict state mutability of some functions. ([#30](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/30))

### Breaking changes
- Requires forge-std version v1.8.0 or later.

## 0.1.0 (2024-03-11)

- Support private networks and forked networks with Defender. ([#25](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/25))

## 0.0.2 (2024-02-20)

- Support constructor arguments for Defender deployments. ([#16](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/16))
- Support Defender deployments for upgradeable contracts. ([#18](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/18))
- Add `Defender.proposeUpgrade` function. ([#21](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/21))
- Add functions to get approval process information from Defender ([#23](https://github.com/OpenZeppelin/openzeppelin-foundry-upgrades/pull/23))

### Breaking changes
- `Defender.deployContract` functions now return `address` instead of `string`.
- Defender deployments now require metadata to be included in compiler output.
- Defender deployments no longer print console output on successful deployments.

## 0.0.1 (2024-02-06)

- Initial preview