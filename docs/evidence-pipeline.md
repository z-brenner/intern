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

## The reply: facts, each with the ids that state it

The prompt (`prompt::build_evidence_prompt`) is laid out like this:

1. The fixed instructions (`EVIDENCE_INSTRUCTIONS`). They are byte-identical
   for every document, and nothing about the document comes before them.
2. One line that says whether the whole document follows or only excerpts.
3. The evidence lines, each written `[handle] text`.

The model never writes the description or the filename. It gives facts
only, and with each fact the ids of the lines that state it:

```json
{"type":"Invoice","type_ids":["p1.b2"],"date":"2025-05-01","date_role":"invoice",
 "date_ids":["p1.b4.f1"],"parties":[{"name":"Halvorsen Fixture Works LLC","role":"issuer",
 "ids":["p1.b1"]},{"name":"Quillon Ridge Bakery, Inc.","role":"customer","ids":["p1.b6.f1"]}],
 "subject":"display shelving","subject_ids":["p1.b7"],"identifier":"INV-10438",
 "identifier_ids":["p1.b3.f1"],"confidence":0.9,"needs_review":false}
```

- **Ids, not quotes.** The model cites a line by its handle. It never copies
  a line back. The engine dereferences each id to the line's own text.
- **A grammar per request** (`prompt::evidence_grammar`). An id can only be
  one of the handles the prompt shows, so the local model cannot cite a
  line it was not shown. A fact always cites at least one id, and an absent
  fact cites none. The handles are written as a prefix tree: as one flat
  alternation, a long document's hundreds of handles cut generation to
  about one token a second in the first live run.
- **Fact first** (`FieldOrder::FactFirst`, the default). Given the ids
  first, the model in the first live run answered the empty list - and with
  it no type, no date and no parties - on every document whose handles were
  quoted ids. Stating a fact and then having to cite at least one line for
  it leaves no such way out. `FieldOrder::EvidenceFirst` is kept so the two
  can be measured.
- **Optional facts are left out, not written as null.** The subject, the
  identifier and the key facts appear only when the lines state them.
- **Caps.** At most 3 parties and 2 key facts. At most 3 ids per fact, and
  2 for a party, the identifier or a key fact. Strings are bounded:
  80 characters for a type, a name or a fact, 100 for the subject and 40
  for the identifier. `StringLimits::Unbounded` drops the string bounds if
  bounded repetition ever costs generation speed.
- **Roles.** A party's role is one of client, contractor, employer,
  employee, buyer, seller, landlord, tenant, issuer, recipient, vendor,
  customer, borrower, lender, licensor, licensee, sender, addressee, or
  other. The prompt tells the model to answer "other" when the lines do
  not say. An invoice lists both its issuer and its customer.
- **The subject** covers the project or matter and the transaction. These
  are not separate fields, which keeps the reply and the grammar small.
- **Id styles.** Handles are either the units' stable ids (`p3.b7.r2`,
  the default) or numbers local to the prompt (`IdStyle::Ordinal`).
  Either way, only stable ids are ever stored.

A hosted model receives the same prompt with no grammar, and its reply is
read leniently (`client::facts_from_text`). The reader accepts either
`type` or `document_type`, one id or a list, and numbers or strings. An id
the prompt did not show is set aside (`ModelFacts::unknown_evidence`),
counted, and is never evidence.

A request with its own grammar is identified by `prompt ‖ \0 ‖ grammar`.
A digest-pipeline request is identified by its prompt alone, as before, so
its recordings stay valid.

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
  filename.
- **Everything else** applies as in the digest pipeline: confidence, the
  model's own request for review, parser warnings, barely readable pages,
  and the token-confidence gate.

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
| Issued | `from` the issuer; for a purchase order, the buyer; then the vendor, seller or sender; then the party the issuer and customer cues name; then the side opposite a supported customer |
| Notice | `for` the party it is about; else `to` the recipient; else `from` the issuer; else `for` the counterpart of a provider; two parties marked other read `between` |
| Letter | `to` the addressee; else `from` the sender; else `for` the party it is about, or the provider's counterpart |
| Email | `from` the sender; else `to` the addressee |
| Record | one party: `for` it if it is the party the record is about, else `from` it. Two or more: `for` the party it is about; else `to` the recipient; else `for` the provider's counterpart; else `from` the issuer |
| Form | `for` the filer (issuer or sender, then employee, tenant, vendor or customer) |
| Unknown | `between` a bilateral pair; else `from` the issuer; else `to` the addressee |

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

Some parts are optional:

- **The amount** is used for issued documents. Elsewhere it is used only
  when labelled as a principal, commitment, price, rent or premium.
- **A key fact** that is not an amount stands in for a missing subject.
- **Too long.** A sentence over 42 words drops parts in a fixed order: the
  amount, the identifier, the preparer, the subject's tail, the subject,
  then the second party. It is never cut mid-phrase.
- **Too short.** A sentence under six words gets the date as the document
  writes it.

The composed sentence then passes the digest pipeline's `validate_description`
unchanged. House style and the firm's own names still apply afterwards, to
the filename and the folder only.

## Running it

| Tool | Flags |
| --- | --- |
| `intern-bench run` | `--pipeline digest\|evidence` `--id-style stable\|ordinal` `--retrieval-tier auto\|whole\|small\|normal\|dense` `--field-order fact-first\|evidence-first` `--string-limits bounded\|unbounded` `--context-tokens N` |
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
