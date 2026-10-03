# Smart Organizer

Smart Organizer is a Windows-first, preview-first desktop file organizer built with Tauri, React, TypeScript, and Rust.

## Current status

The project currently contains:

- A Tauri 2 + React + TypeScript application shell
- Home, Organize, Rules, History, and Settings surfaces
- Rust health/app-info commands plus a read-only first-layer scanner
- A typed scan result with category counts, byte totals, protected-entry counts, and stable ordering
- A centralized category catalog resource reserved for the configurable rules phase
- A typed Zustand UI store
- A local-only visual language with plan-first safety messaging

The current MVP scans only the selected folder's direct file children. It skips directories, hidden/system entries, symbolic links, and special files. Scanning reads metadata only; it never moves, renames, deletes, or overwrites user files.

## Development

```text
pnpm install
pnpm dev
pnpm typecheck
pnpm build
pnpm test
pnpm tauri dev
```

`pnpm dev` runs the browser preview. `pnpm tauri dev` requires the Rust MSVC toolchain and Visual Studio C++ build tools on Windows.

For an offline native verification after dependencies have been fetched:

```text
pnpm tauri build --no-bundle
```

The generated executable is written to `src-tauri/target/release/smart-organizer.exe`. Full MSI/NSIS packaging may require the corresponding WiX/NSIS tool download.

## Safety direction

The application will follow four invariants:

1. Plan before execution.
2. Never overwrite silently.
3. Record every operation.
4. Undo whenever possible.

The Rust side will own scanning, path validation, file operations, transactions, and persistence. React will only present state and request typed commands.
