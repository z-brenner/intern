# Security policy

Intern reads people's documents and renames files on their disks, so a flaw in
it can expose or damage exactly what it was trusted with. Reports are welcome.

## Reporting a vulnerability

Report privately through GitHub Security Advisories:
**[Report a vulnerability](https://github.com/z-brenner/intern/security/advisories/new)**
(the repository's **Security** tab → **Report a vulnerability**).

Please do not open a public issue, pull request, or discussion for a
vulnerability. A private advisory is visible only to you and the maintainer,
and the fix and its disclosure are worked out there.

A useful report says:

- what an attacker can do, and what they need first (a crafted document in a
  watched folder, a position on the network, local access, a shared folder);
- the Intern version, from Settings or the installer name;
- the steps or the file that shows it. Do not attach anyone's real documents:
  a minimal file you made yourself is enough.

## In scope

- The desktop app and its parser worker, including anything a crafted
  document, email, or image can make them do.
- The installer, the update check, and update verification: Intern should
  install nothing that is not signed by the project's key.
- Anything that sends document text, filenames, or identifying data off the
  machine when the README's [What leaves your machine](README.md#what-leaves-your-machine)
  says it should not.
- The shared-folder coordination files, the Microsoft upload verification in
  provisioned builds, and the release pipeline in `.github/workflows/`.
- The download page and guide under `site/`.

The bundled third-party runtimes (llama.cpp, PDFium, Tesseract) are in scope
for how Intern uses them; a flaw in the component itself is best reported
upstream as well.

## Supported versions

Intern is alpha software. Fixes are made on `main` and ship in the next
release; earlier releases are not patched. The in-app update check offers the
fix to installed copies unless it has been switched off.

## Verifying what you downloaded

Every published installer carries a build-provenance attestation. Check that a
file came from this repository's release workflow with:

```sh
gh attestation verify <installer>.exe --repo z-brenner/intern \
  --signer-workflow z-brenner/intern/.github/workflows/release.yml
```
