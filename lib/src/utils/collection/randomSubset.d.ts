/**
 * Picks @param subsetSize elements at random from @param array .
 * The order of the picked elements does not coincide with their
 * order in @param array
 *
 * @param array the array to pick the elements from
 * @param subsetSize the requested size of the subset
 * @param filter called with `(peerInfo)` and should return `true`
 * for every node that should be in the subset
 *
 * @returns array with at most @param subsetSize elements
 * that pass the test.
 *
 * @notice If less than @param subsetSize elements pass the test,
 * the result will contain less than @param subsetSize elements.
 */
export declare function randomSubset<T>(array: T[], subsetSize: number, filter?: (candidate: T) => boolean): T[];
