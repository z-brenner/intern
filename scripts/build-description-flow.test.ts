import { readFile } from 'node:fs/promises';
import { inflateRawSync } from 'node:zlib';
import { describe, expect, it } from 'vitest';
import { buildDescriptionFlow, crc32, packageOutput, storedZip } from './build-description-flow.mjs';

/**
 * The Power Automate package is a download a person imports into their own
 * tenant, where a malformed archive fails with nothing more useful than
 * "something went wrong". Nothing here can import it into a real tenant, so
 * these tests pin what can be pinned locally: the zip is a valid archive read
 * back by an independent reader, it holds exactly the files Power Automate's
 * own exports hold, those files agree with each other about the one SharePoint
 * connection, the flow does what docs/sharepoint-descriptions.md says it does,
 * and the committed download is what the committed source builds.
 */

interface Entry { name: string; data: Buffer; method: number }

/** Reads a zip through its central directory, checking every header and CRC on the way. */
function readZip(zip: Buffer): Entry[] {
  const endOffset = zip.lastIndexOf(Buffer.from([0x50, 0x4b, 0x05, 0x06]));
  expect(endOffset, 'end of central directory record').toBeGreaterThanOrEqual(0);
  expect(endOffset + 22 + zip.readUInt16LE(endOffset + 20), 'nothing after the end record').toBe(zip.length);
  const count = zip.readUInt16LE(endOffset + 10);
  expect(zip.readUInt16LE(endOffset + 8)).toBe(count);
  const directorySize = zip.readUInt32LE(endOffset + 12);
  let cursor = zip.readUInt32LE(endOffset + 16);
  expect(cursor + directorySize).toBe(endOffset);
  const entries: Entry[] = [];
  for (let index = 0; index < count; index += 1) {
    expect(zip.readUInt32LE(cursor), 'central directory header signature').toBe(0x02014b50);
    const method = zip.readUInt16LE(cursor + 10);
    const crc = zip.readUInt32LE(cursor + 16);
    const compressedSize = zip.readUInt32LE(cursor + 20);
    const size = zip.readUInt32LE(cursor + 24);
    const nameLength = zip.readUInt16LE(cursor + 28);
    const extraLength = zip.readUInt16LE(cursor + 30);
    const commentLength = zip.readUInt16LE(cursor + 32);
    const localOffset = zip.readUInt32LE(cursor + 42);
    const name = zip.subarray(cursor + 46, cursor + 46 + nameLength).toString('utf8');
    cursor += 46 + nameLength + extraLength + commentLength;

    expect(zip.readUInt32LE(localOffset), `${name}: local header signature`).toBe(0x04034b50);
    expect(zip.readUInt32LE(localOffset + 14), `${name}: local CRC agrees with the directory`).toBe(crc);
    const localName = zip.subarray(localOffset + 30, localOffset + 30 + zip.readUInt16LE(localOffset + 26)).toString('utf8');
    expect(localName).toBe(name);
    const start = localOffset + 30 + zip.readUInt16LE(localOffset + 26) + zip.readUInt16LE(localOffset + 28);
    const raw = zip.subarray(start, start + compressedSize);
    const data = method === 8 ? inflateRawSync(raw) : Buffer.from(raw);
    expect(data.length, `${name}: size`).toBe(size);
    expect(crc32(data), `${name}: CRC-32`).toBe(crc);
    entries.push({ name, data, method });
  }
  expect(cursor).toBe(endOffset);
  return entries;
}

const FLOW = 'Microsoft.Flow/flows/2fc695c6-70bb-488e-9b34-f962e9aa93bd';

async function built() {
  const entries = readZip(await buildDescriptionFlow());
  const json = (name: string) => JSON.parse(entries.find((entry) => entry.name === name)!.data.toString('utf8'));
  return { entries, json };
}

describe('the zip writer', () => {
  it('computes the standard CRC-32', () => {
    expect(crc32(Buffer.from('123456789'))).toBe(0xcbf43926);
    expect(crc32(Buffer.alloc(0))).toBe(0);
  });

  it('writes an archive its own reader and the format agree on', () => {
    const zip = storedZip([{ name: 'a.txt', data: Buffer.from('alpha') }, { name: 'dir/b.json', data: Buffer.from('{}') }]);
    expect(readZip(zip).map(({ name, data, method }) => [name, data.toString(), method])).toEqual([['a.txt', 'alpha', 0], ['dir/b.json', '{}', 0]]);
  });
});

describe('the description flow package', () => {
  it('holds exactly what a Power Automate package export holds', async () => {
    const { entries } = await built();
    expect(entries.map((entry) => entry.name)).toEqual([
      'manifest.json',
      `${FLOW}/apisMap.json`,
      `${FLOW}/connectionsMap.json`,
      `${FLOW}/definition.json`,
    ]);
    for (const entry of entries) {
      expect(() => JSON.parse(entry.data.toString('utf8')), entry.name).not.toThrow();
      // No byte-order mark: the importer reads these as plain JSON.
      expect(entry.data[0], entry.name).toBe('{'.charCodeAt(0));
    }
  });

  it('names one SharePoint connection, consistently, in every file that refers to it', async () => {
    const { json } = await built();
    const manifest = json('manifest.json');
    const apis = json(`${FLOW}/apisMap.json`);
    const connections = json(`${FLOW}/connectionsMap.json`);
    const definition = json(`${FLOW}/definition.json`);

    const flow = manifest.resources['2fc695c6-70bb-488e-9b34-f962e9aa93bd'];
    expect(flow.type).toBe('Microsoft.Flow/flows');
    expect(flow.suggestedCreationType).toBe('New');
    expect(apis).toEqual({ shared_sharepointonline: expect.any(String) });
    expect(connections).toEqual({ shared_sharepointonline: expect.any(String) });
    expect(manifest.resources[apis.shared_sharepointonline]).toMatchObject({ type: 'Microsoft.PowerApps/apis', id: '/providers/Microsoft.PowerApps/apis/shared_sharepointonline' });
    expect(manifest.resources[connections.shared_sharepointonline]).toMatchObject({ type: 'Microsoft.PowerApps/apis/connections', dependsOn: [apis.shared_sharepointonline] });
    expect([...flow.dependsOn].sort()).toEqual([apis.shared_sharepointonline, connections.shared_sharepointonline].sort());
    expect(Object.keys(manifest.resources)).toHaveLength(3);

    expect(definition.type).toBe('Microsoft.Flow/flows');
    expect(Object.keys(definition.properties.connectionReferences)).toEqual(['shared_sharepointonline']);
    // Every connector step goes through that one reference, so the import asks for exactly one connection.
    const hosts: Array<{ connectionName: string; apiId: string; operationId: string }> = [];
    const visit = (node: unknown) => {
      if (!node || typeof node !== 'object') return;
      const host = (node as { inputs?: { host?: { connectionName: string; apiId: string; operationId: string } } }).inputs?.host;
      if (host) hosts.push(host);
      for (const value of Object.values(node)) visit(value);
    };
    visit(definition.properties.definition);
    expect(hosts.map((host) => host.operationId)).toEqual(['GetOnUpdatedFileItems', 'GetFileContent', 'GetFileMetadataByPath', 'PatchFileItem']);
    for (const host of hosts) expect(host).toMatchObject({ connectionName: 'shared_sharepointonline', apiId: '/providers/Microsoft.PowerApps/apis/shared_sharepointonline' });
  });

  it('runs on Intern records only, and handles a rewritten record as well as a new one', async () => {
    const { json } = await built();
    const { triggers } = json(`${FLOW}/definition.json`).properties.definition;
    const trigger = Object.values(triggers)[0] as { inputs: { host: { operationId: string }; parameters: Record<string, string> }; conditions: Array<{ expression: string }> };
    // Created *or modified*: a backfill rewrites a record in place, and that has to reach the column too.
    expect(trigger.inputs.host.operationId).toBe('GetOnUpdatedFileItems');
    // Site and library are left for the person to choose; no folder, so the dot-folder never has to be found in a picker.
    expect(trigger.inputs.parameters).toEqual({ dataset: '', table: '' });
    expect(trigger.conditions.map((condition) => condition.expression)).toEqual([
      "@endsWith(toLower(triggerOutputs()?['body/{FilenameWithExtension}']), '.json')",
      "@contains(toLower(triggerOutputs()?['body/{Path}']), '.intern/descriptions')",
    ]);
  });

  it('finds the document from the record path, waits for it to sync, and writes the Description column', async () => {
    const { json } = await built();
    const { actions } = json(`${FLOW}/definition.json`).properties.definition;
    expect(actions.Document_path.inputs).toBe("@replace(concat('/', first(split(triggerOutputs()?['body/{Path}'], '.intern/descriptions')), body('Read_the_record')?['path']), '//', '/')");
    expect(actions.Read_the_record.inputs.schema.required).toEqual(['version', 'filename', 'path', 'description']);
    const wait = actions.Wait_for_the_document_to_sync;
    expect(wait.type).toBe('Until');
    expect(wait.actions.Find_the_document.inputs.parameters.path).toBe("@outputs('Document_path')");
    expect(wait.actions.Wait_a_minute.runAfter).toEqual({ Find_the_document: ['Failed', 'TimedOut'] });
    const write = actions.If_the_document_was_found.actions.Write_the_Description_column;
    expect(write.inputs.parameters).toMatchObject({ id: "@variables('DocumentItemId')", 'item/Description': "@body('Read_the_record')?['description']" });
  });

  it('agrees with the record Intern writes', async () => {
    // The fields the flow requires are ones descriptions.rs always writes.
    const source = await readFile('crates/intern-intake/src/descriptions.rs', 'utf8');
    const { json } = await built();
    const schema = json(`${FLOW}/definition.json`).properties.definition.actions.Read_the_record.inputs.schema;
    for (const field of schema.required as string[]) {
      const rust = field.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
      expect(source, field).toMatch(new RegExp(`pub ${rust}: (String|u32),`));
    }
    expect(source).toContain('root.join(".intern").join("descriptions")');
  });

  it('is built deterministically, and the committed download is what the source builds', async () => {
    const first = await buildDescriptionFlow();
    expect((await buildDescriptionFlow()).equals(first)).toBe(true);
    const committed = await readFile(packageOutput);
    expect(committed.equals(first), 'site/downloads/intern-description-flow.zip is stale: run npm run flow:build').toBe(true);
  });
});
