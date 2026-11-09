# Claude integration — deferred

Claude integration is deferred at the maintainer's request. This release contains
no Claude account connection, automatic escalation, browser handoff, or related
settings. Intern analyzes documents locally. The findings below record why the
requested subscription-based automatic fallback was not implemented.

## Why there is no embedded Claude subscription login

Checked 2026-11-09 against Anthropic's official documentation:

- [Authentication and credential use](https://code.claude.com/docs/en/legal-and-compliance#authentication-and-credential-use)
  says third-party developers may not offer Claude.ai login in their applications,
  route requests through subscription credentials on behalf of users, or collect,
  store, or intermediate Claude.ai credentials/session tokens. Developer products
  should use API authentication or a supported cloud provider.
- The same page allows end users to sign into an **unmodified Claude Code binary**
  directly, including on a platform hosting that binary under the stated commercial
  conditions. This does not establish support for using a subscription as Intern's
  unattended inference service.
- [Model configuration](https://code.claude.com/docs/en/model-config#adjust-effort-level)
  describes model-specific effort support. A browser handoff does not control it.

An automatic integration would require a supported developer service and its
billing/account setup, or an agreement with Anthropic. Neither is silently added
as a substitute for the requested subscription connection.

## Retirement of API-key mode

On startup, saved `modelSource: "hosted"` is migrated to `"local"` without
rewriting other settings. If the file cannot be migrated, the application still
reads it with local analysis enforced and preserves the settings recovery UI.
The production pipeline receives the local runtime
rather than the hosted switcher. Hosted key/test/status commands are no longer
registered with the desktop IPC handler. Existing OS-stored keys are not read by
the analysis path. Old hosted library code is retained for compatibility tests,
but the application exposes no API-key option or hosted inference route.
