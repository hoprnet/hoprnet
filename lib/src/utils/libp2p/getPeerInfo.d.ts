import { HoprOptions } from '../../';
import { LevelUp } from 'levelup';
import PeerInfo from 'peer-info';
declare function getPeerInfo(options: HoprOptions, db?: LevelUp): Promise<PeerInfo>;
export { getPeerInfo };
