import { TauriBridge } from '../../lib/tauriBridge';

/*
  Test helpers: queue rows as the desktop backend sends them (QueueItemDto in
  src-tauri/src/commands.rs, filled by queue_item_dto), read through
  TauriBridge as the window reads them. Every field without a skip attribute
  is there, and holds null where the backend has nothing. A status is the
  backend's own word ("needs_review", "applying"), and a reason is the list
  of codes as it sends it ("TYPE_INFERRED, LOW_CONFIDENCE").
*/

export type Sent = Record<string, unknown> & { id: string };

export const sentRow = (id: number, originalFilename: string, status: string, more: Record<string, unknown> = {}): Sent => ({
  id: String(id), originalFilename, status,
  proposedFilename: null, confidence: null, description: null, evidence: null, reason: null, errorCode: null,
  undoable: false, approved: false, proposalRevision: null, reconciliation: null,
  filedPath: null, filedName: null, keptOriginal: false, renamedWithoutApproval: false, parked: false,
  ...more,
});

/** What a reading adds to a row: a name, a sentence, and where in the document each part was found. */
export const sentProposal = (proposedFilename: string, description: string, confidence: number) => ({ proposedFilename, description, confidence, proposalRevision: '1', evidence: { date: null, type: null, parties: null } });

/** A desktop bridge that answers only `queue_list`, with the rows `sent` gives at the moment it is asked. */
export const desktopOver = (sent: () => Sent[]) => new TauriBridge({
  invoke: async <T,>(command: string) => {
    if (command !== 'queue_list') throw new Error(`not played out here: ${command}`);
    return sent().map((row) => ({ ...row })) as T;
  },
  listen: async () => () => {},
});

/** The rows read as the window reads them. */
export const asTheWindowReads = (rows: Sent[]) => desktopOver(() => rows).listItems();
