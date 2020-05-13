"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isBootstrapNode = void 0;
/**
 * Checks whether the given PeerId belongs to any known bootstrap node.
 *
 * @param peerId
 */
function isBootstrapNode(node, peerId) {
    for (let i = 0; i < node.bootstrapServers.length; i++) {
        if (peerId.isEqual(node.bootstrapServers[i].id)) {
            return true;
        }
    }
    return false;
}
exports.isBootstrapNode = isBootstrapNode;
//# sourceMappingURL=isBootstrapNode.js.map