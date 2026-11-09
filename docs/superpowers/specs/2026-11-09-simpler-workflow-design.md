# A simpler Intern workflow

The maintainer requested a complete implementation, reviewed PRs, merge and release, delegating routine design decisions. This document records the implementation direction rather than requesting another design approval.

Everyday work centers on adding documents, reviewing proposed names, and finding completed work. Status moves before the original filename in both lists, leaving the original name, proposed name and description together. Input and Output become Queue folder and Completed folder in user-facing language; existing paths and persisted setting keys remain compatible. Evidence and less common controls remain available through secondary disclosures. Settings expose the common choices first.

The synced-folder approval failure needs a reproduced correction without bypassing source-content or ownership checks. A local-folder experiment succeeded, but the exact client-side cloud failure is not yet observed. Regression tests must distinguish stale enqueue hashes and unreadable sources from files actually changed since extraction.

API-key configuration is retired. Existing hosted configuration must cease sending document text. Anthropic's official rules do not permit a third-party application to offer Claude subscription login or route requests using that subscription; no unsupported connection will be shipped. The maintainer chose to leave Claude out until a supported account connection is available. This release contains no Claude connection or manual handoff.

On the first launch after an update, a brief accessible summary describes the changes in everyday language. Acknowledgement is stored per version, and skipped versions are combined when their notes are available.

Validation includes behavioral regression tests, browser layout/accessibility checks, independent patch review, CI, installed update smoke, whole-product QA, and the existing release evidence/signature/provenance gates. The fresh capture sign-off remains honest and specific to this release.
