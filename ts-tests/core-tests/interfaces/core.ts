import { EventRecord } from "@polkadot/types/interfaces/system";

export interface BlockInterface {
    blockHash: string;
    blockNumber: number;
    events?: Array<EventRecord>;
}
