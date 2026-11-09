import { execFile } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { promisify } from 'node:util';
import { expect, it } from 'vitest';
import { GUIDE_URL, SUPPORT_LINKS } from '../src/lib/bridge';

const exec = promisify(execFile);

it('packages the license directory as a tree so vcpkg subpaths match the signed install manifest', async () => {
  const config = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'));
  expect(config.bundle.windows.nsis.installMode).toBe('currentUser');
  expect(config.bundle.resources['resources/licenses/']).toBe('licenses/');
  expect(config.bundle.resources['resources/licenses/**/*']).toBeUndefined();

  const smoke = await readFile('scripts/smoke-installer.ps1', 'utf8');
  expect(smoke).toContain('$Relative = [string]$Entry.install_path');
  expect(smoke).toContain('Join-Path $InstallDirectory $Relative');
  expect(smoke).toContain('Start-Process -FilePath $App');
  // The close goes to Intern's own window, not Process.MainWindowHandle: with
  // the main window created hidden, that was the single-instance plugin's.
  expect(smoke).toContain('"Tauri Window"');
  expect(smoke).toContain('PostMessage($AppWindow, 0x0010');
  expect(smoke).not.toContain('CloseMainWindow()');
  expect(smoke).toContain('WaitForExit(');
  expect(smoke).toContain('$EvidencePath');
});

// nsis_installer_hooks_configured. "Send to > Intern" is created by the
// installer hooks Tauri includes into its NSIS script, removed again on
// uninstall - but not by the uninstall an update runs, or every update would
// take it away - and checked both ways by the installed smoke on Windows CI.
it('adds Send to > Intern at install and removes it at uninstall, but not during an update', async () => {
  const config = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'));
  expect(config.bundle.windows.nsis.installerHooks).toBe('windows/hooks.nsh');
  // The smoke looks for Intern.lnk; the hooks name it after the product.
  expect(config.productName).toBe('Intern');

  const hooks = await readFile('src-tauri/windows/hooks.nsh', 'utf8');
  const macro = (name: string) => new RegExp(`!macro ${name}\\b([\\s\\S]*?)!macroend`).exec(hooks)?.[1] ?? '';
  const install = macro('NSIS_HOOK_POSTINSTALL');
  expect(install).toContain('CreateShortcut "$SENDTO\\${PRODUCTNAME}.lnk" "$INSTDIR\\${MAINBINARYNAME}.exe"');
  const uninstall = macro('NSIS_HOOK_POSTUNINSTALL');
  expect(uninstall).toContain('${If} $UpdateMode <> 1');
  expect(uninstall).toContain('Delete "$SENDTO\\${PRODUCTNAME}.lnk"');
  expect(uninstall.indexOf('$UpdateMode <> 1')).toBeLessThan(uninstall.indexOf('Delete'));
  // The template's own uninstall already clears the autostart Run value.
  expect(hooks).not.toMatch(/CurrentVersion\\Run/i);

  const smoke = await readFile('scripts/smoke-installer.ps1', 'utf8');
  expect(smoke).toContain('[Environment+SpecialFolder]::SendTo');
  expect(smoke).toContain('Send to shortcut is missing after install');
  expect(smoke).toContain('Send to shortcut remains after uninstall');
  // The window is shown before initialization and a failed start shows it
  // too, so a window alone does not prove the app started: the startup error
  // log is read before the close and again after the exit.
  expect(smoke).toContain('logs/startup-error.log');
  const exited = smoke.indexOf('$AppProcess.ExitCode -ne 0');
  expect(exited).toBeGreaterThan(0);
  expect(smoke.indexOf('Test-Path -LiteralPath $StartupErrorLog', exited)).toBeGreaterThan(exited);
});

// Tauri refuses to build when a plugin's Rust crate and npm package differ in
// major/minor. A caret range on the crate let it float to 2.10.1 against an
// exactly-pinned npm 2.9.0, and the release died at the signing build with
// "Found version mismatched Tauri packages" - after twenty minutes of building
// Tesseract. Both sides are pinned exactly now, and this keeps them together.
it('keeps every Tauri plugin on the same version in Cargo.toml and package.json', async () => {
  const [cargo, packageJson] = await Promise.all([
    readFile('src-tauri/Cargo.toml', 'utf8'),
    readFile('package.json', 'utf8').then(JSON.parse),
  ]);

  const npmPlugins = Object.entries(packageJson.dependencies as Record<string, string>)
    .filter(([name]) => name.startsWith('@tauri-apps/plugin-'))
    .map(([name, version]) => ({ plugin: name.slice('@tauri-apps/plugin-'.length), version }));
  expect(npmPlugins.length).toBeGreaterThan(0);

  for (const { plugin, version } of npmPlugins) {
    const crate = new RegExp(`^tauri-plugin-${plugin}\\s*=\\s*"=?([^"]+)"`, 'm').exec(cargo);
    expect(crate, `tauri-plugin-${plugin} is missing from src-tauri/Cargo.toml`).not.toBeNull();
    // Exact pins on both sides: a range is what allowed the drift.
    expect(version, `@tauri-apps/plugin-${plugin} must be pinned exactly`).toMatch(/^\d+\.\d+\.\d+$/);
    expect(cargo).toContain(`tauri-plugin-${plugin} = "=${version}"`);
    expect(crate![1]).toBe(version);
  }
});

it('leaves packaged-path collision detection to the checker that can actually see the property', async () => {
  const fetch = await readFile('scripts/fetch-windows-assets.ps1', 'utf8');
  // The staged file records are ordered hashtables, and PowerShell cannot
  // resolve a hashtable key as a named property when grouping. Grouping them by
  // the packaged-path key therefore put every file in one unnamed group and
  // reported a collision for any package holding more than one file. That check
  // never passed, and it is redundant: verify-assets.mjs performs it with real
  // property access and is covered by verify-assets.test.ts.
  expect(fetch).not.toMatch(/Group-Object\s+install_path/);
  expect(fetch).toMatch(/node .*verify-assets\.mjs.* --require-bundled/);
  // The manifest must be written before it is verified, or the check reads stale
  // contents. Match the invocation, not the comment above it.
  expect(fetch.indexOf('$Manifest.bundled_files = $BundledFiles'))
    .toBeLessThan(fetch.lastIndexOf('verify-assets.mjs'));
});

it('keeps the release verifier outside Tauri binary discovery and invokes its workspace tool', async () => {
  const [{ stdout }, release] = await Promise.all([
    exec('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1']),
    readFile('.github/workflows/release.yml', 'utf8'),
  ]);
  const metadata = JSON.parse(stdout);
  const app = metadata.packages.find((pkg: { name: string }) => pkg.name === 'intern-app');
  const verifier = metadata.packages.find((pkg: { name: string }) => pkg.name === 'intern-release-verifier');
  expect(app).toBeDefined();
  expect(verifier).toBeDefined();
  const binaries = app.targets.filter((target: { kind: string[] }) => target.kind.includes('bin')).map((target: { name: string }) => target.name);
  expect(binaries).toEqual(['intern']);
  expect(binaries).not.toContain('verify-updater-artifact');
  expect(verifier.targets.filter((target: { kind: string[] }) => target.kind.includes('bin')).map((target: { name: string }) => target.name)).toEqual(['intern-release-verifier']);
  expect(release).toContain('cargo run --locked -p intern-release-verifier --');
  expect(release).not.toContain('-p intern-app --bin verify-updater-artifact');
});

// The webview may ask the shell to open a URL only through the opener plugin,
// and the capability scope is what actually confines it. It names exactly the
// addresses the bridge hands over - the guide site and the two fixed support
// links outside it - so a new link cannot be added on one side without the
// other. The description-flow link is a section of the guide, inside the
// guide site's entry.
it('scopes the opener capability to the guide and the fixed support links only', async () => {
  const capability = JSON.parse(await readFile('src-tauri/capabilities/default.json', 'utf8'));
  const opener = (capability.permissions as Array<string | { identifier: string; allow?: Array<{ url: string }> }>)
    .filter((permission) => (typeof permission === 'string' ? permission : permission.identifier).startsWith('opener:'));

  expect(opener).toEqual([{
    identifier: 'opener:allow-open-url',
    allow: [
      { url: 'https://z-brenner.github.io/intern/*' },
      { url: SUPPORT_LINKS['sharepoint-site'] },
      { url: SUPPORT_LINKS['onedrive-download'] },
    ],
  }]);
  expect(GUIDE_URL.startsWith('https://z-brenner.github.io/intern/')).toBe(true);
  expect(SUPPORT_LINKS['description-flow'].startsWith('https://z-brenner.github.io/intern/')).toBe(true);
});

// A Linux package keeps llama-server, the parser worker, and everything they
// load in one runtime folder under /usr/lib/Intern rather than in /usr/bin,
// where llama-server and tesseract would collide with the distribution's own
// and the CPU backends could not be found beside the server. What it does not
// carry - Tesseract, the OpenMP runtime, and the OpenSSL llama-server is
// linked against (libssl3t64 on Ubuntu 24.04, libssl3 before it) - it depends
// on.
it('bundles the Linux runtime as one folder and leaves the Windows layout to Windows', async () => {
  const [base, linux] = await Promise.all([
    readFile('src-tauri/tauri.conf.json', 'utf8').then(JSON.parse),
    readFile('src-tauri/tauri.linux.conf.json', 'utf8').then(JSON.parse),
  ]);
  expect(base.bundle.targets).toEqual(['nsis']);
  expect(linux.bundle.targets).toEqual(['deb']);
  expect(linux.bundle.externalBin).toEqual([]);
  expect(linux.bundle.resources['resources/runtime/']).toBe('runtime/');
  // Merged as a JSON merge patch: null removes a Windows mapping, anything
  // not named is kept as the base config has it. Everything under native/ is
  // Windows-only, upstream llama.cpp's folder included.
  expect(base.bundle.resources['resources/native/llama-upstream/']).toBe('llama-upstream/');
  for (const source of Object.keys(base.bundle.resources)) {
    const windowsOnly = /\.(dll|traineddata|onnx)$/.test(source) || source.startsWith('resources/native/');
    expect(linux.bundle.resources[source], source).toBe(windowsOnly ? null : undefined);
  }
  expect(linux.bundle.linux.deb.depends).toEqual(['libgomp1', 'libssl3t64 | libssl3', 'tesseract-ocr']);
  // The license tree ships on Linux under the same path as on Windows.
  expect(base.bundle.resources['resources/licenses/']).toBe('licenses/');
});

// The Linux counterpart of Send to > Intern: a launcher that accepts files is
// one a file manager offers under Open With, and one documents can be dropped
// on. Tauri's own template starts Intern with no files at all.
it('lets a Linux file manager open documents with Intern', async () => {
  const linux = JSON.parse(await readFile('src-tauri/tauri.linux.conf.json', 'utf8'));
  const template = await readFile(`src-tauri/${linux.bundle.linux.deb.desktopTemplate}`, 'utf8');
  expect(template).toMatch(/^Exec=\{\{exec\}\} %F$/m);
  // No MimeType of its own: declaring PDF or Word documents would offer Intern
  // as their default application, and opening a document is not filing it.
  expect(template).not.toMatch(/^MimeType=(?!\{\{mime_type\}\})/m);
});

// The llama.cpp release zip is flat: llama-server sits beside every CLI tool,
// their impl DLLs and an RPC backend. The Windows package takes the server's
// import closure and every CPU backend variant - ggml picks the one the
// processor can run at start-up - and nothing else, into llama-upstream\, with
// the OpenMP runtime its ggml-base and CPU backends import. It is the one
// llama.cpp build: no llama-server is a sidecar at the install root any more.
it('stages exactly one llama-server, its import closure and the CPU backend variants, in llama-upstream', async () => {
  const [fetch, config, manifest] = await Promise.all([
    readFile('scripts/fetch-windows-assets.ps1', 'utf8'),
    readFile('src-tauri/tauri.conf.json', 'utf8').then(JSON.parse),
    readFile('src-tauri/resources/runtime-assets.json', 'utf8').then(JSON.parse),
  ]);
  const closure = ['llama-server-impl.dll', 'llama-common.dll', 'mtmd.dll', 'llama.dll', 'ggml.dll', 'ggml-base.dll'];
  const required = /\$UpstreamRequired = @\(([^)]*)\)/.exec(fetch)?.[1];
  expect(required?.split(',').map((name) => name.trim().replace(/"/g, ''))).toEqual(closure);
  expect(fetch).toContain('$LlamaUpstreamDirectory = Join-Path $NativeDirectory "llama-upstream"');
  expect(fetch).toContain('$UpstreamServer = Join-Path $LlamaUpstreamDirectory "llama-server.exe"');
  expect(fetch).toMatch(/Install-LlamaServer -Archive \$Downloads\["llama\.cpp-upstream"\] .*-ServerDestination \$UpstreamServer -Folder "llama-upstream" -Required \$UpstreamRequired$/m);
  expect(fetch.match(/Install-LlamaServer -Archive/g)).toHaveLength(1);
  expect(fetch).toContain('return "llama-upstream/$(Split-Path -Leaf $Path)"');
  expect(fetch).toContain('-Filter "ggml-cpu-*.dll"');
  expect(fetch).toContain('carries no CPU backend');
  expect(fetch).not.toMatch(/-notmatch '-impl\\\.dll\$'/);
  // No RPC backend, and no other build's OpenMP runtime.
  for (const unwanted of ['ggml-rpc', 'libomp140']) expect(fetch).not.toContain(`"${unwanted}`);
  // No llama-server sidecar: not in the bundle, not pinned, and one an earlier
  // staging left in binaries\ is removed before the inventory is taken.
  expect(config.bundle.externalBin).toEqual(['binaries/intern-worker', 'binaries/tesseract']);
  expect(manifest.downloads.map((download: { id: string }) => download.id).filter((id: string) => id.startsWith('llama.cpp')).sort())
    .toEqual(['llama.cpp-license', 'llama.cpp-upstream']);
  expect(fetch).toContain('$StaleLlamaSidecar = Join-Path $BinariesDirectory "llama-server-$TargetTriple.exe"');
  expect(fetch.indexOf('Remove-Item -LiteralPath $StaleLlamaSidecar')).toBeLessThan(fetch.indexOf('$BundledFiles = @('));

  // libomp.dll ships beside upstream's server, owned by LLVM's OpenMP
  // package rather than llama.cpp, with its license.
  const code = fetch.split('\n').filter((line) => !line.trimStart().startsWith('#')).join('\n');
  expect(code).toContain('$OpenMpRuntime = Join-Path $UpstreamDirectory "libomp.dll"');
  expect(code).toContain('Copy-SidecarDll $OpenMpRuntime "llvm-openmp" $UpstreamVersion -Folder "llama-upstream"');
  expect(code.match(/"llvm-openmp"/g)).toHaveLength(1);
  expect(code.match(/Copy-SidecarDll \$OpenMpRuntime\b/g)).toHaveLength(1);
  for (const [literal] of code.matchAll(/"[^"\n]*libomp[^"\n]*"/g)) expect(literal).toMatch(/\blibomp\.dll"$/);
  expect(code).toContain('$OpenMpLicense = Join-Path $UpstreamDirectory "LICENSE-LLVM-OpenMP"');
  expect(code).toContain('Copy-RuntimeFile $OpenMpLicense (Join-Path $LicensesDirectory "llvm-openmp-LICENSE.txt")');
});

// The installed package must carry the server, and it must start from where
// it is installed: a process cannot start at all while a DLL it imports is
// missing from beside it, and upstream's ggml-base imports libomp.dll.
it('requires the installed llama-server in llama-upstream and starts it from there', async () => {
  const smoke = await readFile('scripts/smoke-installer.ps1', 'utf8');
  const required = /foreach \(\$Required in @\(([^)]*)\)\)/.exec(smoke)?.[1]
    .split(',').map((name) => name.trim().replace(/"/g, ''));
  expect(required).toEqual(expect.arrayContaining([
    'llama-upstream/llama-server.exe', 'llama-upstream/libomp.dll', 'llama-upstream/ggml-base.dll',
  ]));
  expect(required).not.toContain('llama-server.exe');
  expect(smoke).toContain('foreach ($ServerRelative in @("llama-upstream/llama-server.exe"))');
  expect(smoke).toContain('-ArgumentList "--version" -WorkingDirectory (Split-Path -Parent $ServerPath)');
  expect(smoke).toContain('if ($ServerProcess.ExitCode -ne 0) { throw');
  // Its output stays out of the installation, which the uninstall check
  // further down requires to be empty again.
  expect(smoke).toContain('$ServerOutput = Join-Path ([IO.Path]::GetTempPath())');
  expect(smoke.indexOf('"--version"')).toBeLessThan(smoke.indexOf('$Uninstaller = '));
});

// Releases up to alpha.14 installed a llama.cpp build at the install root, and
// an update installs over the old files without uninstalling them, so the
// installer's pre-install hook removes that build by name. The names are the
// ones the releases' published inventories list at the root as llama.cpp's (or
// the OpenMP runtime alpha.11 carried with it), and the smoke checks the same
// list is gone after it has installed - over such a release, when asked.
const OLD_ROOT_LLAMA_FILES = [
  'llama-server.exe', 'llama-server-impl.dll', 'llama-common.dll', 'llama.dll', 'mtmd.dll', 'ggml.dll', 'ggml-base.dll', 'ggml-rpc.dll', 'libomp140.x86_64.dll',
  ...['x64', 'sse42', 'ssse3', 'sandybridge', 'ivybridge', 'piledriver', 'haswell', 'alderlake', 'skylakex', 'cannonlake', 'cascadelake', 'cooperlake', 'icelake', 'zen4', 'sapphirerapids']
    .map((variant) => `ggml-cpu-${variant}.dll`),
];

it('removes the root llama.cpp build of releases up to alpha.14 before an install, by name, only over Intern', async () => {
  const hooks = await readFile('src-tauri/windows/hooks.nsh', 'utf8');
  const preinstall = /!macro NSIS_HOOK_PREINSTALL\b([\s\S]*?)!macroend/.exec(hooks)?.[1] ?? '';
  const code = preinstall.split('\n').filter((line) => !line.trimStart().startsWith(';')).join('\n');
  // Over an existing install only: a first install into a folder of someone's
  // own deletes nothing.
  expect(code).toMatch(/^\s*\$\{If\} \$\{FileExists\} "\$INSTDIR\\\$\{MAINBINARYNAME\}\.exe"$/m);
  const deleted = [...code.matchAll(/^\s*Delete "\$INSTDIR\\([^"\\]+)"$/gm)].map((match) => match[1]);
  expect(deleted.sort()).toEqual([...OLD_ROOT_LLAMA_FILES].sort());
  // Nothing else: no pattern, no folder, nothing under llama-upstream\.
  expect(code.match(/\bDelete\b/g)).toHaveLength(OLD_ROOT_LLAMA_FILES.length);
  expect(code).not.toMatch(/\*|\?|RMDir|llama-upstream|\/r\b|\/REBOOTOK/i);

  const smoke = await readFile('scripts/smoke-installer.ps1', 'utf8');
  const listed = /\$OldRootLlamaFiles = @\(([\s\S]*?)\)\n/.exec(smoke)?.[1]
    .split(',').map((name) => name.trim().replace(/"/g, ''));
  expect(listed?.sort()).toEqual([...OLD_ROOT_LLAMA_FILES].sort());
  expect(smoke).toContain('if (Test-Path -LiteralPath (Join-Path $InstallDirectory $OldRootFile)) { throw');
  // An update runs the installer over the earlier release the way the in-app
  // updater does: silently, with /UPDATE, and never uninstalls first.
  expect(smoke).toContain('[string]$PreviousInstallerPath');
  expect(smoke).toContain('$InstallArguments = "/S /UPDATE"');
  expect(smoke).toContain('Start-Process -FilePath $Installer -ArgumentList $InstallArguments -Wait -PassThru');
  expect(smoke.indexOf('$PreviousInstall = Start-Process')).toBeLessThan(smoke.indexOf('$Install = Start-Process'));
});
