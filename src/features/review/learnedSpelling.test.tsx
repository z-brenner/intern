import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { ReviewInspector } from '../../components/ReviewInspector';
import { UseOnItsOwnQuestion, typedOut } from '../../components/UseOnItsOwn';
import { openRow } from '../queue/gridTesting';
import { clearField, commitComposition, composeInto, fillWithoutTyping, insertAtOnce, typeInto } from './typingTesting';
import { createInMemoryBridge, createLearnedSpellingBridge } from '../../lib/inMemoryBridge';
import { humanizeReason } from '../../lib/reasons';
import type { HouseRule, LearnedRule, QueueItem } from '../../types';

const selectRow = openRow;

const respelled: QueueItem = {
  id: 'sow',
  originalFilename: 'SOW final v3.pdf',
  status: 'ready',
  proposedFilename: '2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso.pdf',
  confidence: 0.93,
  description: 'Statement of work between Ridgeline Cartography LLC and Contoso Worldwide, Inc. for the 2026 member-map engagement.',
  evidence: { date: 'effective as of April 1, 2026', type: 'STATEMENT OF WORK', parties: 'Ridgeline Cartography LLC; Contoso Worldwide, Inc.' },
  houseRules: [{ kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }],
};

describe('a name in the reviewer\'s own spelling', () => {
  it('says which spelling was applied while the evidence keeps the document\'s words', async () => {
    render(<App bridge={createInMemoryBridge({ items: [respelled] })} />);
    await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));

    const note = screen.getByRole('note', { name: 'Learned spellings applied' });
    expect(note).toHaveTextContent('Uses your spelling: Contoso Worldwide, Inc. written as Contoso. Change or forget it under Settings.');
    expect(screen.getByLabelText('Filename')).toHaveValue('2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso');
    fireEvent.click(screen.getByText('Analysis details'));
    const evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(within(evidence).getByText('Contoso Worldwide, Inc.')).toBeVisible();
  });

  // A spelling ending in its own period ("Inc.") is the end of the list:
  // the sentence takes no second one. It read "Receiver, Inc.. Change or
  // forget it under Settings."
  it('ends the note with one period after a spelling that ends in one', async () => {
    const abbreviated: QueueItem = { ...respelled, houseRules: [{ kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso, Inc.' }] };
    render(<App bridge={createInMemoryBridge({ items: [abbreviated] })} />);
    await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));

    const note = screen.getByRole('note', { name: 'Learned spellings applied' });
    expect(note).toHaveTextContent('Uses your spelling: Contoso Worldwide, Inc. written as Contoso, Inc. Change or forget it under Settings.');
    expect(note.textContent).not.toContain('..');
  });

  it('shows no note for a name in the document\'s own words', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    expect(screen.queryByRole('note', { name: 'Learned spellings applied' })).not.toBeInTheDocument();
  });
});

function handlers() {
  return {
    onClose: vi.fn(), onApprove: vi.fn(), onKeep: vi.fn(), onCancel: vi.fn(), onRetry: vi.fn(),
    onReanalyze: vi.fn(), onRemove: vi.fn(), onUndo: vi.fn(), onOpen: vi.fn(), onReveal: vi.fn(),
  };
}

/** A statement of work held for a spelling nobody has checked, as the backend lists it. */
const held: QueueItem = {
  ...respelled,
  status: 'review',
  errorCode: 'LEARNED_SPELLING_UNCONFIRMED',
  reasonCodes: ['LEARNED_SPELLING_UNCONFIRMED'],
  reason: humanizeReason('LEARNED_SPELLING_UNCONFIRMED'),
  proposedFilename: '2026-04-01 SOW between Ridgeline Cartography and Contoso.pdf',
  houseRules: [
    { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' },
    { kind: 'party', from: 'Ridgeline Cartography LLC', to: 'Ridgeline Cartography' },
    { kind: 'document_type', from: 'Statement of Work', to: 'SOW' },
  ],
};

/** The learned spellings: the first two the held name used, unchecked; the third used and checked; one more unchecked that it did not use. */
const spellings: LearnedRule[] = [
  { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso', seen: 2, active: true, confirmed: 0, learnedAt: 1 },
  { id: '2', kind: 'party', from: 'Ridgeline Cartography LLC', to: 'Ridgeline Cartography', seen: 2, active: true, confirmed: 0, learnedAt: 2 },
  { id: '3', kind: 'document_type', from: 'Statement of Work', to: 'SOW', seen: 2, active: true, confirmed: 1, learnedAt: 3 },
  { id: '4', kind: 'party', from: 'Northwind Traders', to: 'Northwind', seen: 2, active: true, confirmed: 0, learnedAt: 4 },
];

/** A bridge holding `rules`, whose check behaves as the backend's: exact, in use, or refused. */
function spellingBridge(rules: LearnedRule[]) {
  let current = rules.map((rule) => ({ ...rule }));
  const houseRulesList = vi.fn(async () => current.map((rule) => ({ ...rule })));
  const houseRuleCheck = vi.fn(async (id: string, shown: HouseRule) => {
    const rule = current.find((candidate) => candidate.id === id);
    if (!rule) throw { code: 'RULE_NOT_FOUND', message: 'learned spelling does not exist' };
    if (!rule.active || rule.kind !== shown.kind || rule.from !== shown.from || rule.to !== shown.to) throw { code: 'RULE_CHANGED', message: 'learned spelling is not in use as it was shown' };
    current = current.map((candidate) => candidate.id === id ? { ...candidate, confirmed: candidate.confirmed + 1 } : candidate);
  });
  return { houseRulesList, houseRuleCheck, respell(id: string, to: string) { current = current.map((rule) => rule.id === id ? { ...rule, to } : rule); } };
}

/** A held statement of work whose name used `rules`, unchecked. */
const heldFor = (rules: LearnedRule[]): QueueItem => ({ ...held, houseRules: rules.map(({ kind, from, to }) => ({ kind, from, to })) });

const CONTOSO = 'Use on its own: Contoso Worldwide, Inc. written as Contoso';
const RIDGELINE = 'Use on its own: Ridgeline Cartography LLC written as Ridgeline Cartography';

/** The question open for the spelling `label` names: the group, the field the spelling is typed into, its yes and its Cancel. */
function question(label: string) {
  const group = screen.getByRole('group', { name: label });
  return {
    group,
    field: within(group).getByRole('textbox'),
    yes: within(group).getByRole('button', { name: 'Use on its own' }),
    cancel: within(group).getByRole('button', { name: 'Cancel' }),
  };
}

/** A press on `button`, as Chromium makes one: the pointer down, then a click counting one press. */
const pressOn = (button: HTMLElement) => { fireEvent.pointerDown(button, { pointerType: 'mouse' }); fireEvent.click(button, { detail: 1 }); };

// No approval checks a learned spelling. A document held for one shows,
// beside "Uses your spelling", "Use on its own" for each spelling nobody has
// checked that its name used - the only way to stop being asked - after the
// question Settings asks. It checks the spelling, as shown, and nothing else:
// the document is neither approved nor filed.
describe('using a held document\'s spelling on its own', () => {
  afterEach(() => cleanup());

  it('is offered only for the unchecked spellings the name used, beside the pointer to Settings', async () => {
    const bridge = spellingBridge(spellings);
    render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} />);
    expect(await screen.findByRole('button', { name: CONTOSO })).toBeVisible();
    expect(screen.getByRole('button', { name: RIDGELINE })).toBeVisible();
    // Not the checked one, nor one the name did not use.
    expect(screen.getAllByRole('button', { name: /^Use on its own:/ })).toHaveLength(2);
    expect(screen.getByRole('note', { name: 'Learned spellings applied' })).toHaveTextContent('Change or forget it under Settings.');
  });

  it('is not offered for a document not held for a spelling', async () => {
    const bridge = spellingBridge(spellings);
    const { rerender } = render(<ReviewInspector item={{ ...held, status: 'ready', errorCode: undefined, reasonCodes: undefined, reason: undefined }} drawer={false} {...handlers()} spellings={bridge} />);
    rerender(<ReviewInspector item={{ ...held, errorCode: 'LOW_CONFIDENCE', reasonCodes: ['LOW_CONFIDENCE'], reason: humanizeReason('LOW_CONFIDENCE') }} drawer={false} {...handlers()} spellings={bridge} />);
    await Promise.resolve();
    expect(screen.queryByRole('button', { name: /Use on its own/ })).not.toBeInTheDocument();
    expect(bridge.houseRulesList).not.toHaveBeenCalled();
  });

  it('asks first, for the spelling to be typed, and Cancel or Escape take the question back and nothing else', async () => {
    const bridge = spellingBridge(spellings);
    const actions = handlers();
    render(<ReviewInspector item={held} drawer {...actions} spellings={bridge} />);
    fireEvent.click(await screen.findByRole('button', { name: CONTOSO }));
    const asked = question(CONTOSO);
    expect(asked.group).toHaveAccessibleDescription('Every later document Intern reads as naming Contoso Worldwide, Inc. will name Contoso instead, and may be filed under that name without your check. Do this only if that is right for every one of them. Choosing this does not approve this document: it still waits for you.');
    expect(asked.field).toHaveAccessibleName('Type Contoso to use it on its own');
    expect(asked.field).toHaveFocus();
    expect(asked.field).toHaveValue('');
    expect(asked.yes).toBeDisabled();

    fireEvent.click(asked.cancel);
    expect(screen.queryByRole('group', { name: /^Use on its own:/ })).not.toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('button', { name: CONTOSO })).toHaveFocus());

    fireEvent.click(screen.getByRole('button', { name: CONTOSO }));
    fireEvent.keyDown(question(CONTOSO).field, { key: 'Escape' });
    expect(screen.queryByRole('group', { name: /^Use on its own:/ })).not.toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('button', { name: CONTOSO })).toHaveFocus());
    // The drawer stays open: Escape took back the question only.
    expect(actions.onClose).not.toHaveBeenCalled();
    expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
  });

  it('checks the spelling as shown once it is typed, and neither approves nor files the document', async () => {
    const bridge = spellingBridge(spellings);
    const actions = handlers();
    const onSpellingChecked = vi.fn();
    render(<ReviewInspector item={held} drawer={false} {...actions} spellings={bridge} onSpellingChecked={onSpellingChecked} />);
    fireEvent.click(await screen.findByRole('button', { name: RIDGELINE }));
    const { field, yes } = question(RIDGELINE);
    typeInto(field, 'Ridgeline Cartography');
    fireEvent.click(yes);
    await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledWith('2', { kind: 'party', from: 'Ridgeline Cartography LLC', to: 'Ridgeline Cartography' }));
    expect(await screen.findByRole('status')).toHaveTextContent('Intern will use “Ridgeline Cartography” on its own from now on. This document still waits for you.');
    await waitFor(() => expect(screen.queryByRole('button', { name: /Use on its own: Ridgeline/ })).not.toBeInTheDocument());
    expect(screen.getByRole('button', { name: /Use on its own: Contoso/ })).toBeVisible();
    expect(onSpellingChecked).toHaveBeenCalledOnce();
    expect(actions.onApprove).not.toHaveBeenCalled();
    expect(bridge.houseRuleCheck).toHaveBeenCalledOnce();
    await waitFor(() => expect(screen.getByRole('group', { name: 'Use a learned spelling on its own' })).toHaveFocus());
  });

  // Every rule that tried to tell a press meant for the yes from one that
  // was not - no double-click, half a second's wait, a pointer that moved
  // onto it - was got round: a slow double-click, a scroll, a tap through a
  // menu, a window closing over it. The yes now waits for the spelling to
  // be typed out, which no press does, and which makes the person read it.
  describe('answers only once the spelling is typed out', () => {
    it('keeps its yes disabled, and takes no click on it nor Enter in the field, until then', async () => {
      const bridge = spellingBridge(spellings);
      const actions = handlers();
      render(<ReviewInspector item={held} drawer={false} {...actions} spellings={bridge} />);
      fireEvent.click(await screen.findByRole('button', { name: CONTOSO }));
      const { group, field, yes } = question(CONTOSO);
      for (const typed of ['', 'Contos', 'Contosoo', 'Cantoso', 'Contoso Ltd']) {
        clearField(field);
        typeInto(field, typed);
        expect(field, typed).toHaveValue(typed);
        expect(yes, typed).toBeDisabled();
        // A press, a click from the keyboard or assistive technology, and Enter.
        fireEvent.pointerDown(yes, { pointerType: 'mouse' });
        fireEvent.click(yes, { detail: 1 });
        fireEvent.click(yes, { detail: 0 });
        fireEvent.keyDown(field, { key: 'Enter' });
        expect(group, typed).toBeInTheDocument();
      }
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();

      clearField(field);
      typeInto(field, 'Contoso');
      expect(yes).toBeEnabled();
      fireEvent.keyDown(field, { key: 'Enter' });
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledWith('1', { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }));
      expect(bridge.houseRuleCheck).toHaveBeenCalledOnce();
      expect(actions.onApprove).not.toHaveBeenCalled();
    });

    it('is the spelling\'s letters and digits in order, whatever the case, accents, spaces and punctuation', () => {
      expect(typedOut('contoso ltd', 'Contoso Ltd.')).toBe(true);
      expect(typedOut('CONTOSO-LTD', 'Contoso Ltd.')).toBe(true);
      expect(typedOut('Muller', 'Müller')).toBe(true);
      expect(typedOut('Ｃｏｎｔｏｓｏ', 'Contoso')).toBe(true);
      expect(typedOut('山田太郎', '山田太郎')).toBe(true);
      expect(typedOut('3M', '3M Company')).toBe(false);
      expect(typedOut('Mller', 'Müller')).toBe(false);
      expect(typedOut('Mueller', 'Müller')).toBe(false);
      expect(typedOut('Contoso Ltd. 2', 'Contoso Ltd.')).toBe(false);
      expect(typedOut('Ltd. Contoso', 'Contoso Ltd.')).toBe(false);
      expect(typedOut('山田', '山田太郎')).toBe(false);
      expect(typedOut('', 'Contoso')).toBe(false);
      expect(typedOut(' .', 'Contoso')).toBe(false);
      // A spelling with no letter or digit is typed exactly, spaces at its ends aside.
      expect(typedOut(' & ', '&')).toBe(true);
      expect(typedOut('+', '&')).toBe(false);
      expect(typedOut('', ' ')).toBe(false);
      expect(typedOut(' ', ' ')).toBe(false);
    });

    // An accent may be left off, as a keyboard without it does. A mark that
    // makes another letter or syllable may not: typing the document's own
    // カトウ confirmed ガトウ, and राम confirmed रमा.
    it('lets go of accents, and of no mark that makes another letter', () => {
      expect(typedOut('Nguyen', 'Nguyễn')).toBe(true);
      expect(typedOut('Dvorak', 'Dvořák')).toBe(true);
      expect(typedOut('ELENI', 'ΕΛΕΝΗ')).toBe(false);
      expect(typedOut('Ελενη', 'Ελένη')).toBe(true);
      expect(typedOut('ΕΛΕΝΗΣ', 'Ελένης')).toBe(true);
      expect(typedOut('Ελένησ', 'Ελένης')).toBe(true);
      // Hebrew and Arabic are written without their vowel points as a rule.
      expect(typedOut('שלום', 'שָׁלוֹם')).toBe(true);
      expect(typedOut('محمد', 'مُحَمَّد')).toBe(true);
      // Elsewhere a mark is a letter's own.
      expect(typedOut('ガトウ', 'ガトウ')).toBe(true);
      expect(typedOut('ｶﾞﾄｳ', 'ガトウ')).toBe(true);
      expect(typedOut('カトウ', 'ガトウ')).toBe(false);
      expect(typedOut('さとう', 'さどう')).toBe(false);
      expect(typedOut('ホンタ', 'ホンダ')).toBe(false);
      expect(typedOut('ポンダ', 'ホンダ')).toBe(false);
      expect(typedOut('राम', 'रमा')).toBe(false);
      expect(typedOut('कमल', 'कमला')).toBe(false);
      expect(typedOut('करन', 'किरन')).toBe(false);
      expect(typedOut('कषितिज', 'क्षितिज')).toBe(false);
      expect(typedOut('क्षितिज', 'क्षितिज')).toBe(true);
      expect(typedOut('Иошкар', 'Йошкар')).toBe(false);
      expect(typedOut('Ежов', 'Ёжов')).toBe(false);
      expect(typedOut('Йошкар', 'Йошкар')).toBe(true);
      expect(typedOut('สดา', 'สุดา')).toBe(false);
      expect(typedOut('கம', 'காம')).toBe(false);
    });

    // A keyboard without ø, ß or ł types o, ss and l, and the spelling's
    // symbols are none of its letters: "Acme" is "Acme™" as it is "Acme®".
    it('takes a letter a keyboard may lack as it is typed without it, and leaves out what is no letter as written', () => {
      for (const [typed, spelling] of [
        ['Moller & Son A/S', 'Møller & Søn A/S'], ['Soren', 'Søren'], ['Strasse', 'Straße'], ['STRASSE', 'Straße'], ['strasse', 'STRAẞE'],
        ['Lodz', 'Łódź'], ['Dorde Dordevic', 'Đorđe Đorđević'], ['Caesar', 'Cæsar'], ['Aero', 'Ærø'], ['Oeuvre', 'Œuvre'],
        ['Kirikkale', 'Kırıkkale'], ['KIRSEHIR', 'Kırşehir'], ['Thor', 'Þór'], ['Gudmundsdottir', 'Guðmundsdóttir'], ['Hamrun', 'Ħamrun'],
        ['Acme', 'Acme™'], ['Acme', 'Acme®'], ['Acme', 'Acme℠'], ['finance', 'ﬁnance'], ['Hawaii', 'Hawaiʻi'], ['Abd al-Aziz', 'ʿAbd al-ʿAzīz'],
        ['葛飾', '葛\u{E0100}飾'],
      ]) expect(typedOut(typed, spelling), `${typed} for ${spelling}`).toBe(true);
      for (const [typed, spelling] of [
        ['Moler', 'Møller'], ['Mpller', 'Møller'], ['Stase', 'Straße'], ['Strase', 'Straße'], ['Acmetm', 'Acme™'], ['Acm', 'Acme™'],
        ['Contoso Ltd', 'Contoso'], ['Contos', 'Contoso'], ['Thr', 'Þór'],
      ]) expect(typedOut(typed, spelling), `${typed} for ${spelling}`).toBe(false);
      // A spelling that is all symbols is typed as it is.
      expect(typedOut('™', '™')).toBe(true);
      expect(typedOut('tm', '™')).toBe(false);
      expect(typedOut('', '™')).toBe(false);
    });

    it('takes the spelling typed in any case, with or without its accents, spaces and punctuation, and not a letter missing or added', async () => {
      const rules: LearnedRule[] = [
        { id: '7', kind: 'party', from: 'Contoso Limited', to: 'Contoso Ltd.', seen: 2, active: true, confirmed: 0, learnedAt: 7 },
        { id: '8', kind: 'party', from: 'Mueller GmbH', to: 'Müller', seen: 2, active: true, confirmed: 0, learnedAt: 8 },
      ];
      const bridge = spellingBridge(rules);
      render(<ReviewInspector item={heldFor(rules)} drawer={false} {...handlers()} spellings={bridge} />);
      for (const [rule, right, wrong] of [[rules[0], 'contoso ltd', ['contoso lt', 'contoso ltdd']], [rules[1], 'Muller', ['Mller', 'Mueller']]] as const) {
        const label = `Use on its own: ${rule.from} written as ${rule.to}`;
        fireEvent.click(await screen.findByRole('button', { name: label }));
        const { field, yes } = question(label);
        for (const typed of wrong) {
          clearField(field);
          typeInto(field, typed);
          expect(yes, typed).toBeDisabled();
        }
        clearField(field);
        typeInto(field, right);
        expect(yes, right).toBeEnabled();
        fireEvent.click(yes);
        await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenLastCalledWith(rule.id, { kind: rule.kind, from: rule.from, to: rule.to }));
      }
    });

    it('takes nothing into its field but typing: no paste, drop, suggestion, undo or autofill, and asks the browser for no suggestions', async () => {
      const bridge = spellingBridge(spellings);
      render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} />);
      fireEvent.click(await screen.findByRole('button', { name: CONTOSO }));
      const { field, yes, cancel } = question(CONTOSO);
      expect(fillWithoutTyping(field, 'Contoso')).toEqual([]);
      expect(field).toHaveValue('');
      expect(yes).toBeDisabled();
      expect(field).toHaveAttribute('autocomplete', 'off');
      expect(field).toHaveAttribute('autocorrect', 'off');
      expect(field).toHaveAttribute('autocapitalize', 'off');
      expect(field).toHaveAttribute('spellcheck', 'false');
      // A name of its own, so nothing typed into another is suggested for it.
      const name = field.getAttribute('name');
      fireEvent.click(cancel);
      fireEvent.click(await screen.findByRole('button', { name: RIDGELINE }));
      expect(question(RIDGELINE).field.getAttribute('name')).not.toBe(name);
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    it('takes a name typed with an input method, and not the Enter that ends the composition', async () => {
      const rules: LearnedRule[] = [{ id: '9', kind: 'party', from: 'Yamada Taro', to: '山田太郎', seen: 2, active: true, confirmed: 0, learnedAt: 9 }];
      const bridge = spellingBridge(rules);
      render(<ReviewInspector item={heldFor(rules)} drawer={false} {...handlers()} spellings={bridge} />);
      const label = 'Use on its own: Yamada Taro written as 山田太郎';
      fireEvent.click(await screen.findByRole('button', { name: label }));
      const { field, yes } = question(label);
      composeInto(field, ['やまだ', '山田', '山田たろう', '山田太郎']);
      expect(field).toHaveValue('山田太郎');
      expect(yes).toBeEnabled();
      commitComposition(field);
      // WebKit ends the composition before the Enter that commits it, a key it marks as the input method's.
      fireEvent.keyDown(field, { key: 'Enter', keyCode: 229 });
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      expect(question(label).group).toBeInTheDocument();
      fireEvent.keyDown(field, { key: 'Enter' });
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledWith('9', { kind: 'party', from: 'Yamada Taro', to: '山田太郎' }));
    });

    // A key types one character. A word put in at once, with no keys for
    // it - an input method's clipboard, an on-screen keyboard's, an emoji
    // keyboard's - was taken as typed, and one press checked the spelling.
    // An input method that commits what its keys spelled, without showing a
    // composition, puts in no more than the keys pressed for it.
    it('takes no more characters at once than keys were pressed for them', async () => {
      const rules: LearnedRule[] = [
        { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso', seen: 2, active: true, confirmed: 0, learnedAt: 1 },
        { id: '9', kind: 'party', from: 'Yamada Taro', to: '山田太郎', seen: 2, active: true, confirmed: 0, learnedAt: 9 },
      ];
      const bridge = spellingBridge(rules);
      render(<ReviewInspector item={heldFor(rules)} drawer={false} {...handlers()} spellings={bridge} />);
      fireEvent.click(await screen.findByRole('button', { name: CONTOSO }));
      const { field, yes, cancel } = question(CONTOSO);
      for (const keys of [0, 1, 2, 6]) {
        insertAtOnce(field, 'Contoso', keys);
        expect(field, `${keys} keys`).toHaveValue('');
        expect(yes, `${keys} keys`).toBeDisabled();
      }
      // Keys pressed before a character went in are spent on it.
      for (let key = 0; key < 7; key += 1) fireEvent.keyDown(field, { key: 'Process', keyCode: 229 });
      typeInto(field, 'C');
      insertAtOnce(field, 'ontoso', 0);
      expect(field).toHaveValue('C');
      clearField(field);
      insertAtOnce(field, 'Contoso', 7);
      expect(field).toHaveValue('Contoso');
      expect(yes).toBeEnabled();
      fireEvent.click(cancel);

      const label = 'Use on its own: Yamada Taro written as 山田太郎';
      fireEvent.click(await screen.findByRole('button', { name: label }));
      // "yamadatarou" and the key that commits it.
      insertAtOnce(question(label).field, '山田太郎', 12);
      expect(question(label).field).toHaveValue('山田太郎');
      fireEvent.click(question(label).yes);
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledWith('9', { kind: 'party', from: 'Yamada Taro', to: '山田太郎' }));
      expect(bridge.houseRuleCheck).toHaveBeenCalledOnce();
    });

    it('starts empty every time it asks', async () => {
      render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={spellingBridge(spellings)} />);
      fireEvent.click(await screen.findByRole('button', { name: CONTOSO }));
      typeInto(question(CONTOSO).field, 'Contoso');
      expect(question(CONTOSO).yes).toBeEnabled();
      fireEvent.keyDown(question(CONTOSO).field, { key: 'Escape' });
      fireEvent.click(await screen.findByRole('button', { name: CONTOSO }));
      expect(question(CONTOSO).field).toHaveValue('');
      expect(question(CONTOSO).yes).toBeDisabled();
      typeInto(question(CONTOSO).field, 'Contoso');
      fireEvent.click(question(CONTOSO).cancel);
      fireEvent.click(await screen.findByRole('button', { name: RIDGELINE }));
      expect(question(RIDGELINE).field).toHaveValue('');
      expect(question(RIDGELINE).yes).toBeDisabled();
    });

    it('starts empty again when the spelling it asks about changes while it asks', () => {
      const onConfirm = vi.fn();
      type Shown = Pick<LearnedRule, 'id' | 'kind' | 'from' | 'to'>;
      const rule: Shown = { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' };
      const ask = (shown: Shown) => <UseOnItsOwnQuestion rule={shown} describedBy="asked" onConfirm={onConfirm} onCancel={() => undefined} />;
      const { rerender } = render(ask(rule));
      typeInto(screen.getByRole('textbox'), 'Contoso');
      expect(screen.getByRole('button', { name: 'Use on its own' })).toBeEnabled();
      const changes: Shown[] = [{ ...rule, id: '5' }, { ...rule, kind: 'document_type' }, { ...rule, from: 'Contoso Worldwide' }, { ...rule, to: 'Contoso Ltd.' }];
      for (const changed of changes) {
        rerender(ask(changed));
        expect(screen.getByRole('textbox'), JSON.stringify(changed)).toHaveValue('');
        expect(screen.getByRole('textbox')).toHaveFocus();
        expect(screen.getByRole('button', { name: 'Use on its own' })).toBeDisabled();
        typeInto(screen.getByRole('textbox'), changed.to);
        expect(screen.getByRole('button', { name: 'Use on its own' })).toBeEnabled();
      }
      expect(onConfirm).not.toHaveBeenCalled();
    });

    // The review window's shortcuts work from its own fields: Ctrl+Enter
    // approves and Alt+K keeps from the name, Alt+N moves on. Not from this
    // one, where a person types a spelling and nothing else.
    it('keeps every key typed in its field to itself: no approving, keeping or moving on from there', async () => {
      const base = createLearnedSpellingBridge();
      const bridge = { ...base, approve: vi.fn(base.approve), keepOriginal: vi.fn(base.keepOriginal), houseRuleCheck: vi.fn(base.houseRuleCheck) };
      render(<App bridge={bridge} />);
      const panel = await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));
      fireEvent.click(await within(panel).findByRole('button', { name: CONTOSO }));
      const { field } = question(CONTOSO);
      for (const typed of ['Cont', 'oso']) {
        typeInto(field, typed);
        for (const key of [
          { key: 'Enter', ctrlKey: true }, { key: 'Enter', metaKey: true }, { key: 'k', code: 'KeyK', altKey: true },
          { key: 'n', code: 'KeyN', altKey: true }, { key: 'F2' }, { key: 'Delete' }, { key: '/' }, { key: 'j' },
        ]) fireEvent.keyDown(field, key);
      }
      expect(field).toHaveFocus();
      expect(field).toHaveValue('Contoso');
      expect(panel).toHaveTextContent('SOW final v3.pdf');
      expect(bridge.approve).not.toHaveBeenCalled();
      expect(bridge.keepOriginal).not.toHaveBeenCalled();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    it('takes itself back on Escape from its field, and lets Tab through, but not the Escape an input method takes', () => {
      const onCancel = vi.fn();
      render(<UseOnItsOwnQuestion rule={{ id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }} describedBy="asked" onConfirm={vi.fn()} onCancel={onCancel} />);
      const field = screen.getByRole('textbox');
      const beyond = vi.fn();
      document.addEventListener('keydown', beyond);
      try {
        fireEvent.keyDown(field, { key: 'Escape', isComposing: true });
        expect(onCancel).not.toHaveBeenCalled();
        fireEvent.keyDown(field, { key: 'Escape' });
        expect(onCancel).toHaveBeenCalledOnce();
        expect(beyond).not.toHaveBeenCalled();
        // Tab goes on to the drawer's or the dialog's own keys, which keep the focus in them.
        fireEvent.keyDown(field, { key: 'Tab' });
        expect(beyond).toHaveBeenCalledOnce();
      } finally {
        document.removeEventListener('keydown', beyond);
      }
    });
  });

  // Opened by a press, the question opens where its action was, sentence
  // first, and nothing moves under the pointer: the panel used to scroll it
  // clear of the decision, which brought its buttons under the pointer for
  // the next press. Opened from the keyboard, it is brought into sight.
  describe('opened by a press, moves nothing under the pointer', () => {
    afterEach(() => { vi.restoreAllMocks(); Reflect.deleteProperty(Element.prototype, 'scrollIntoView'); });

    it('takes the focus in its field without scrolling when a press opened it, and is brought into sight when a key did', async () => {
      const focus = vi.spyOn(HTMLElement.prototype, 'focus');
      // jsdom scrolls nothing, and has no scrollIntoView to call.
      const scrollIntoView = vi.fn();
      Object.defineProperty(Element.prototype, 'scrollIntoView', { value: scrollIntoView, configurable: true, writable: true });
      render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={spellingBridge(spellings)} />);
      pressOn(await screen.findByRole('button', { name: CONTOSO }));
      const pressed = question(CONTOSO);
      expect(pressed.field).toHaveFocus();
      expect(focus.mock.calls.filter((_, index) => focus.mock.contexts[index] === pressed.field)).toEqual([[{ preventScroll: true }]]);
      expect(scrollIntoView).not.toHaveBeenCalled();
      fireEvent.keyDown(pressed.field, { key: 'Escape' });

      const again = await screen.findByRole('button', { name: CONTOSO });
      fireEvent.keyDown(again, { key: 'Enter' });
      fireEvent.click(again, { detail: 0 });
      const keyed = question(CONTOSO);
      expect(keyed.field).toHaveFocus();
      expect(scrollIntoView.mock.contexts).toEqual([keyed.group]);
      expect(scrollIntoView).toHaveBeenCalledWith({ block: 'nearest' });
    });

    // A press on the sentence clicks nothing in the question, and leaves the
    // focus on the group around it. Escape there closed the drawer, question
    // and all.
    it('takes back the question, and only the question, on Escape wherever the focus is in the drawer', async () => {
      const bridge = spellingBridge(spellings);
      const actions = handlers();
      render(<ReviewInspector item={held} drawer {...actions} spellings={bridge} />);
      for (const focusElsewhere of [
        () => screen.getByRole('group', { name: 'Use a learned spelling on its own' }).focus(),
        () => screen.getByLabelText('Filename').focus(),
        () => question(CONTOSO).yes.focus(),
      ]) {
        pressOn(await screen.findByRole('button', { name: CONTOSO }));
        const asked = question(CONTOSO);
        typeInto(asked.field, 'Contoso');
        fireEvent.pointerDown(asked.group.querySelector('p')!, { pointerType: 'mouse' });
        focusElsewhere();
        fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
        expect(asked.group).not.toBeInTheDocument();
        await waitFor(() => expect(screen.getByRole('button', { name: CONTOSO })).toHaveFocus());
      }
      expect(actions.onClose).not.toHaveBeenCalled();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // With no question open, Escape closes the drawer as before.
      fireEvent.keyDown(screen.getByRole('button', { name: CONTOSO }), { key: 'Escape' });
      expect(actions.onClose).toHaveBeenCalledOnce();
    });
  });

  // The record's spelling can be older than the spelling: an approval of
  // another document can respell it while this one is on screen. Checking a
  // spelling the person was not shown would let Intern file names they
  // never agreed to.
  it('says when the spelling changed since it was shown, and checks nothing', async () => {
    const bridge = spellingBridge(spellings);
    const onSpellingChecked = vi.fn();
    render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} onSpellingChecked={onSpellingChecked} />);
    fireEvent.click(await screen.findByRole('button', { name: CONTOSO }));
    bridge.respell('1', 'Contoso Ltd');
    typeInto(question(CONTOSO).field, 'Contoso');
    fireEvent.click(question(CONTOSO).yes);
    expect(await screen.findByRole('alert')).toHaveTextContent('This spelling changed - look at it again.');
    expect(bridge.houseRuleCheck).toHaveBeenCalledWith('1', { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' });
    expect(onSpellingChecked).not.toHaveBeenCalled();
    // As it reads now, it is not the spelling the name used: nothing to offer for it.
    await waitFor(() => expect(screen.queryByRole('button', { name: /Use on its own: Contoso/ })).not.toBeInTheDocument());
  });

  // The note above lists every spelling the name used. A bare "Use on its
  // own" under it - the only unchecked one of three, or the one left after
  // the other was checked and named in the line below - read as an action
  // for whichever spelling was nearest. Each says which it is on screen
  // whenever the note lists more than one.
  it('names the spelling beside "Use on its own" when it is the one unchecked of several the note lists', async () => {
    const oneUnchecked = spellingBridge(spellings.map((rule) => rule.id === '2' ? { ...rule, confirmed: 1 } : rule));
    render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={oneUnchecked} />);
    await screen.findByRole('button', { name: CONTOSO });
    const group = screen.getByRole('group', { name: 'Use a learned spelling on its own' });
    expect(within(group).getAllByRole('button', { name: /^Use on its own:/ })).toHaveLength(1);
    expect(group).toHaveTextContent('Contoso Worldwide, Inc. written as Contoso');
  });

  it('names the spelling beside the "Use on its own" left once the other was checked', async () => {
    const bridge = spellingBridge(spellings);
    render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} />);
    fireEvent.click(await screen.findByRole('button', { name: CONTOSO }));
    typeInto(question(CONTOSO).field, 'Contoso');
    fireEvent.click(question(CONTOSO).yes);
    await screen.findByRole('status');
    await waitFor(() => expect(screen.queryByRole('button', { name: /Use on its own: Contoso/ })).not.toBeInTheDocument());
    const left = screen.getByRole('group', { name: 'Use a learned spelling on its own' });
    expect(within(left).getAllByRole('button', { name: /^Use on its own:/ })).toHaveLength(1);
    expect(left).toHaveTextContent('Ridgeline Cartography LLC written as Ridgeline Cartography');
  });

  it('shows the action alone under a note of one spelling', async () => {
    const one: QueueItem = { ...held, houseRules: [held.houseRules![0]] };
    render(<ReviewInspector item={one} drawer={false} {...handlers()} spellings={spellingBridge(spellings)} />);
    await screen.findByRole('button', { name: CONTOSO });
    expect(screen.getByRole('group', { name: 'Use a learned spelling on its own' })).toHaveTextContent(/^Use on its own$/);
  });

  it('is offered from the review window of the app, which reads the spellings from the bridge', async () => {
    const base = createInMemoryBridge({ items: [held], learnedRules: spellings });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    render(<App bridge={{ ...base, houseRuleCheck }} />);
    await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));
    fireEvent.click(await screen.findByRole('button', { name: CONTOSO }));
    typeInto(question(CONTOSO).field, 'Contoso');
    fireEvent.click(question(CONTOSO).yes);
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('1', { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }));
    expect((await base.houseRulesList()).find((rule) => rule.id === '1')?.confirmed).toBe(1);
  });
});
