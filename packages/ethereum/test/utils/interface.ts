/**
 * copyied from OpenZeppelin's test ``SupportsInterface.behavior.js``
 * https://github.com/OpenZeppelin/openzeppelin-contracts/blob/9eba3ef29a252eaa760a1f5afb281412e15d4529/test/utils/introspection/SupportsInterface.behavior.js
 */
import { expect } from 'chai'
import { makeInterfaceId } from '@openzeppelin/test-helpers'
import { Contract } from 'ethers'

const INTERFACES = {
  ERC165: ['supportsInterface(bytes4)'],
  ERC721: [
    'balanceOf(address)',
    'ownerOf(uint256)',
    'approve(address,uint256)',
    'getApproved(uint256)',
    'setApprovalForAll(address,bool)',
    'isApprovedForAll(address,address)',
    'transferFrom(address,address,uint256)',
    'safeTransferFrom(address,address,uint256)',
    'safeTransferFrom(address,address,uint256,bytes)'
  ],
  ERC721Enumerable: ['totalSupply()', 'tokenOfOwnerByIndex(address,uint256)', 'tokenByIndex(uint256)'],
  ERC721Metadata: ['name()', 'symbol()', 'tokenURI(uint256)'],
  ERC1155: [
    'balanceOf(address,uint256)',
    'balanceOfBatch(address[],uint256[])',
    'setApprovalForAll(address,bool)',
    'isApprovedForAll(address,address)',
    'safeTransferFrom(address,address,uint256,uint256,bytes)',
    'safeBatchTransferFrom(address,address,uint256[],uint256[],bytes)'
  ],
  ERC1155Receiver: [
    'onERC1155Received(address,address,uint256,uint256,bytes)',
    'onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)'
  ],
  AccessControl: [
    'hasRole(bytes32,address)',
    'getRoleAdmin(bytes32)',
    'grantRole(bytes32,address)',
    'revokeRole(bytes32,address)',
    'renounceRole(bytes32,address)'
  ],
  AccessControlEnumerable: ['getRoleMember(bytes32,uint256)', 'getRoleMemberCount(bytes32)'],
  IHoprBoost: ['boostOf(uint256)', 'typeIndexOf(uint256)']
}

const INTERFACE_IDS = {}
const FN_SIGNATURES = {}

for (const k of Object.getOwnPropertyNames(INTERFACES)) {
  INTERFACE_IDS[k] = makeInterfaceId.ERC165(INTERFACES[k])
  for (const fnName of INTERFACES[k]) {
    // the interface id of a single function is equivalent to its function signature
    FN_SIGNATURES[fnName] = makeInterfaceId.ERC165([fnName])
  }
}

export const shouldSupportInterfaces = (contract: Contract, interfaces: any[]) => {
  describe('Contract interface', function () {
    for (const k of interfaces) {
      const interfaceId = INTERFACE_IDS[k]
      describe(k, function () {
        describe("ERC165's supportsInterface(bytes4)", function () {
          it('uses less than 30k gas [skip-on-coverage]', async function () {
            expect(await contract.estimateGas.supportsInterface(interfaceId)).to.be.lte(30000)
          })

          if (k !== 'IHoprBoost') {
            it('claims support', async function () {
              expect(await contract.supportsInterface(interfaceId)).to.equal(true)
            })
          }
        })

        for (const fnName of INTERFACES[k]) {
          const fnSig = FN_SIGNATURES[fnName]
          describe(fnName, function () {
            it('has to be implemented', function () {
              expect(
                contract.interface.fragments.filter((fn) =>
                  !fn.name || fn.type !== 'function' ? '' : contract.interface.getSighash(fn.format()) === fnSig
                ).length
              ).to.equal(1)
            })
          })
        }
      })
    }
  })
}
