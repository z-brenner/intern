# The evidence pipeline

The evidence pipeline reads a document a second way. It is opt-in
(`Engine::with_pipeline(Pipeline::Evidence)`, `--pipeline evidence` in the
evaluation tools) until a live evaluation shows it does at least as well
as the digest pipeline. The digest pipeline stays the default, and both
replay gates run on it unchanged.

```text
DocumentSource ─▶ index ─▶ retrieve ─▶ prompt + grammar ─▶ one inference
               ─▶ facts with evidence ids ─▶ validate ─▶ compose ─▶ name
```

[Evidence retrieval](evidence-retrieval.md) describes the index and the
retrieval. This page covers the parts after them: the reply, its
validation, and the filename and description composed from it.

## The reply: facts, each with the id of its line

The model never writes the description or the filename. It gives facts
only, and with each fact the id of the line that states it. The reply has
two forms (`prompt::ReplyForm`).

### The compact reply (the default)

```json
{"type":["Invoice","p1.b2"],"date":["2025-05-01","invoice","p1.b4.f1"],
 "parties":[["Halvorsen Fixture Works LLC","issuer","p1.b1"],
            ["Quillon Ridge Bakery, Inc.","customer","p1.b6.f1"]],
 "subject":["display shelving","p1.b7"]}
```

- **Each fact is an array with its id last**: the type, the date with its
  role, up to three parties with their roles, and optionally a subject.
  An absent type or date is `null`.
- **No amount.** The amount is read from the document: an issued
  document's last labelled total (`Total`, `Amount Due`, `Balance Due`),
  otherwise the first amount a field or table row labels as a price,
  principal, salary, rent, fee or similar, otherwise the first total. An
  amount's id cost 9 to 10 generated tokens (`,"amount":"p1.b9.r2"`) and
  named the line the document's own label finds.
- **No confidence, no review flag, no identifier, no key facts.** In the
  96 live replies of the first form the reply's own confidence was 0.9 and
  its `needs_review` false every time, so neither gated anything; offered
  as an optional `"review":true`, the flag was raised on 5 of 19 fixtures
  the gold expects ready. 14 of 77 identifiers were an id written back. The
  identifier is read from the document instead: the number after the type
  on its title line (`PACKING SLIP PS-311`), or a field labelled as that
  kind of document's number (`Invoice No.:`, `Policy Number`).
- **The instructions are the system turn** (`COMPACT_INSTRUCTIONS`, 418
  tokens with the model's tokenizer; 460 with the shared opening line)
  and the user turn is the document alone: one line that says whether
  the whole document follows or only excerpts, then the evidence lines,
  each written `[handle] text`.
  llama-server keeps a hybrid model's cache only at checkpoints - the start
  of the last user message, and a few tokens before the end of the prompt -
  so instructions at the head of the user turn were prefilled afresh for
  every document (`cached_tokens` 46). In the system turn they are the same
  prefix for every document, before the user message's checkpoint, and are
  read once. `ModelRequest::system` carries the turn; a request's identity
  covers it.
- **Strings start with a letter or a digit** (or any non-ASCII character),
  so a value is never a placeholder such as `..`, which the first compact
  run copied from its skeleton as the type of 11 of 19 fixtures. Strings are
  bounded: 80 characters for a type or a name, 60 for the subject.

### The fields reply (`--reply-form facts`)

The form the first live recordings were made with: named fields and id
lists, a subject, an identifier, up to two key facts, a confidence and a
review flag, after instructions at the head of the user turn
(`EVIDENCE_INSTRUCTIONS`). It keeps its prompt version (`6134f0168dcf`), so
those recordings still replay.

```json
{"type":"Invoice","type_ids":["p1.b2"],"date":"2025-05-01","date_role":"invoice",
 "date_ids":["p1.b4.f1"],"parties":[{"name":"Halvorsen Fixture Works LLC","role":"issuer",
 "ids":["p1.b1"]}],"subject":"display shelving","subject_ids":["p1.b7"],
 "identifier":"INV-10438","identifier_ids":["p1.b3.f1"],"confidence":0.9,"needs_review":false}
```

`FieldOrder::EvidenceFirst` puts each fact's ids before it in this form.
Given the ids first, the model in the first live run answered the empty
list - and with it no type, no date and no parties - on every document
whose handles were quoted ids; fact first is the default.

### Both forms

- **Ids, not quotes.** The model cites a line by its handle and never copies
  it back. The engine dereferences each id to the line's own text.
- **A grammar per request** (`prompt::evidence_grammar`). An id can only be
  one of the handles the prompt shows, so the local model cannot cite a
  line it was not shown. The handles are written as a prefix tree: as one
  flat alternation, a long document's hundreds of handles cut generation to
  about one token a second.
- **Roles.** A party's role is one of client, contractor, employer,
  employee, buyer, seller, landlord, tenant, issuer, recipient, vendor,
  customer, borrower, lender, licensor, licensee, sender, addressee, or
  other.
- **Id styles.** Handles are either the units' stable ids (`p3.b7.r2`,
  the default) or numbers local to the prompt (`IdStyle::Ordinal`).
  Either way, only stable ids are ever stored.

A hosted model receives the same prompt with no grammar, and its reply is
read leniently (`client::facts_from_text`): either form, one id or a list,
numbers or strings, an array's parts told apart by what they are. An id
the prompt did not show is set aside (`ModelFacts::unknown_evidence`),
counted, and is never evidence.

A request with its own grammar is identified by `prompt ‖ \0 ‖ grammar`,
and its own system turn after that. A digest-pipeline request is
identified by its prompt alone, as before, so its recordings stay valid.

## Validation: what the model was shown, and the whole document

`facts::ValidationScope` reads a reply against two views of the document.

- **context**: the units the prompt carried. A fact is accepted only if
  the context states it, the same way the digest pipeline accepts only
  what its digest states. A fact the document states only outside the
  excerpts is not accepted, because the model could not have read it.
- **document**: every unit. The guards that turn a date away run over both
  views, and either one firing is enough:
  - the date belongs to another document;
  - the date is a deadline;
  - the day and month could be read either way round.

  A replacement date is taken only when both views agree on a single one.

Each fact records its `Support`:

- `Cited`: a unit the reply cited for the fact states it.
- `Context`: only another excerpt states it. This is the support the
  digest pipeline accepts.
- `Unsupported`: nothing the model was shown states it.
- `Absent`: the reply did not state the fact.

A cited id whose unit does not state its fact (`miscited_ids`) and an id
the prompt never showed (`unknown_ids`) are counted. Neither is ever
evidence.

The other checks:

- **Free text.** The names and numbers in the subject, the identifier and
  the key facts are checked the way a description's claims are. One that
  is not supported is left out of the description and sends the document
  to review (`DESCRIPTION_UNSUPPORTED`).
- **Subject wording.** A subject is written into the description only when
  at least 60% of its words are in the units it cites.
- **Roles.** A role is supported when the document states it next to the
  name:
  - a label (`Bill To:`, `Landlord:`, `| Tenant |`, `Dear`);
  - a defined term (`("Tenant")`, `, as Lender`);
  - a page-one letterhead, for an issuer or a sender;
  - the issuer and customer cues on the same line.

  A role nothing supports never decides how the parties read in the
  filename. When the reply gives a party no supported role, the role the
  document's own wording gives it does (`ValidatedParty::document_role`):
  `Resident:` before the name makes a tenant, `Owner:` a landlord,
  `("Lender")` after it a lender.
- **No value.** A placeholder (`..`, `null`) or an evidence id written
  where a value belongs (`p1.b1`) is no value: neither accepted nor sent to
  review as an unsupported claim.
- **The type** is the document's own phrase. When the reply cites a title
  line that names the same kind of document, the line's phrase is the type
  ("Loan Notice" citing `NOTICE OF DEFAULT AND RESERVATION OF RIGHTS` is a
  Notice of Default and Reservation of Rights). Otherwise the reply's words
  must be stated whole somewhere in the context, and not as another
  document's name (`Re: Residential Lease Agreement dated ...`), and must
  name a kind of document. A type the document does not state never
  reaches the name or the description: a title of the same kind stands in,
  for review (`TYPE_INFERRED`), or the type is unsupported.
- **Names.** A party's name loses a field's label, a street address or a
  second name that the layout ran into it (`PrOperty 47 JUniper LOOP Cedar
  Finch Properties Llc`); a legal form ends an organisation's name. A name
  that is the end of an organisation a unit names (`MANUFACTURING LLC` of
  `EMBer POSt MANUFACtURInG LLC`) is completed to it. Capitals OCR
  scattered are read as capitals. A first name on its own is unsupported.
- **Who is a party.** Someone named only on a `cc:` line is copied in, and
  someone who signs for an organisation among the parties (a name, a job
  title, then the organisation) acts for it: both stay on record, with role
  other, and neither is ever a filename's party.
- **Roles** a party keeps are the ones the document supports: the reply's,
  else the one the document's wording gives, else none
  (`ValidatedParty::proposed_role` keeps what the reply said).
- **The date.** An agreement's or a form's signed-on date yields to the one
  effective or start date both views agree on.
- **Everything else** applies as in the digest pipeline: the model's own
  request for review (a hosted reply can still make one), parser warnings,
  barely readable pages, and the token-confidence gate.

The evidence a reviewer sees, in `Evidence` and in `ValidatedFacts::evidence`,
is always a line of the document, dereferenced by the engine. The UI's
evidence rows need no change.

`validate.rs`'s tests also run every digest-based case against the
index's units, as the context of a document sent whole, and require the
same outcome (see the `validate` wrapper in its test module).

## Composition

`compose.rs` builds the filename's parties and joining word, and the
description, from the validated facts.

### Document class

`DocumentClass::of` sorts the validated type into one of these classes,
tested in this order:

1. amendment
2. notice
3. issued (invoices, receipts, orders, quotes, statements, slips, bills of
   lading, rate confirmations, price lists)
4. agreement
5. email
6. letter
7. form
8. record
9. unknown

Families whose names contain another family's words are tested first. A
statement of work is an agreement, a policy's declarations are a record,
and an addendum is an amendment.

### The relation table

`relation_from_roles` uses supported roles only. Parties are taken in the
order the document first names them.

| Class | Relation |
| --- | --- |
| Agreement, amendment | `between` the first two parties; one party, `with` |
| Issued | `from` the issuer; for a purchase order, the buyer; then the vendor, seller or sender; then the party the issuer and customer cues name, or the one party at the head of the first page before any customer cue; then the side opposite a supported customer. The customer alone is never the filename's party. A reply that names no one takes the one organisation at the head of the first page as the issuer |
| Notice | `for` the party it is about; else `to` the recipient; else `from` the issuer; else `for` the counterpart of a provider; else `from` the provider; two parties marked other read `between` |
| Letter | `to` the addressee; else `from` the sender; else `for` the party it is about, or the provider's counterpart; else `from` the provider |
| Email | `from` the sender; else `to` the addressee |
| Record | one party: `for` it if it is the party the record is about, else `from` it. Two or more: `for` the party it is about; else `to` the recipient; else `for` the provider's counterpart; else `from` the issuer |
| Form | `between` the two parties the document names "by and between"; else `for` the filer (issuer or sender, then employee, tenant, vendor or customer) |
| Unknown | `between` a bilateral pair, or the two named "by and between"; else `from` the issuer; else `to` the addressee |

"The party it is about" means a client, customer, tenant, employee,
borrower, licensee or buyer. When nothing settles the relation, the first
party is kept with no joining word (`Invoice - Acme`). A relation is never
guessed.

`tests/relation_gold.rs` checks the table against every InternBench gold
document. Each party is given every role its gold allows, in both orders
of appearance, and the derived relation must be one the gold accepts. The
one exception is a party with no role whose only counterpart is "other".
There, nothing decides the relation, and the table keeps the name with no
joining word.

### The description

Each class has its own template, filled with validated facts only, in the
document's own spellings:

| Class | Template |
| --- | --- |
| Agreement | `{type} between {A} and {B} for {subject}.` |
| Issued | `{type} from {issuer} to {customer} for {subject}, {invoice} {number}, totalling {amount}.` |
| Notice, letter | `{type} from {issuer} to {party} regarding {subject}.` |
| Email | `{type} from {sender} to {addressee} about {subject}.` |
| Record | `{type} for {party} covering {subject}, prepared by {issuer}.` |
| Form | `{type} for {filer} submitted to {counterparty} regarding {subject}.` |

Values, not labels:

- **The subject** is a value. A labelled field gives its value (`Project:
  Aurora Catalog Project`); a field that names a party (`Bill to ...`), a
  subject that only repeats the type or names a party or the identifier, is
  left out. A run of capitalised words the document never states as one
  phrase is written in lower case.
- **The amount** reads with the label the document gives it: `totalling
  $1,248.00` for an issued document's total, `annual fee of $96,000` for a
  labelled one, beside the subject it is the price of (`for ... ($312,500)`)
  or `for $X` otherwise.
- **The identifier** stands after the type when it is on the type's title
  line (`Invoice INV-2048 from ...`), and reads with its label elsewhere
  (`policy KC-WC-7710345`). An identifier without a digit is not a number.
- **A key fact** of the fields reply that is not an amount stands in for a
  missing subject.
- **The type** is the document's whole title; a compound title too long for
  the filename to name both parties keeps its first kind there
  ("Settlement Agreement" of "Settlement Agreement and Mutual Release")
  before the filename drops a party.
- **Too long.** A sentence over 42 words drops parts in a fixed order: the
  preparer, the identifier, the subject's tail, the amount, the subject,
  then the second party. It is never cut mid-phrase.
- **Too short.** A sentence under six words gets the date as the document
  writes it.

The composed sentence then passes the digest pipeline's `validate_description`
unchanged. House style and the firm's own names still apply afterwards, to
the filename and the folder only.

## Running it

| Tool | Flags |
| --- | --- |
| `intern-bench run` | `--pipeline digest\|evidence` `--id-style stable\|ordinal` `--retrieval-tier auto\|whole\|small\|normal\|dense` `--reply-form compact\|facts` `--field-order fact-first\|evidence-first` (fields reply only) `--string-limits bounded\|unbounded` `--context-tokens N` |
| `intern-evaluate` | the same flags; `--pipeline new` (the default) and `legacy` keep their meaning |

A recording made with the evidence pipeline carries extra fields in its
header:

- `pipeline`;
- the retrieval configuration's fingerprint;
- the evidence prompt's version (`prompt::evidence_prompt_version`).

Replay refuses a recording made by the other pipeline. When the retrieval
or the prompt has changed, replay says so, the same way it reports a
changed digest budget. Every document whose prompt the change affects is
then `stale_prompt`.

InternBench records these extra scores for the evidence pipeline only:

| Score | Meaning |
| --- | --- |
| `facts_proposed` | facts the reply stated |
| `facts_unsupported` | of those, the facts nothing supported |
| `cited_support_rate` | the share of accepted facts that a cited unit supported |
| `unknown_ids` | cited ids the prompt never showed |
| `miscited_ids` | cited ids whose unit does not state the fact |
