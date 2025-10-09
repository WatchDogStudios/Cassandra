# Windows Build Setup for CassandraNet

The current build failure (`LNK1104: cannot open file 'msvcrt.lib'`) indicates the MSVC Universal CRT libraries are missing from your Visual Studio Build Tools installation.

## Required Components
Install / Modify Visual Studio 2022 Build Tools with these workloads:
- "Desktop development with C++" (core)

And these individual components (tick explicitly if customizing):
- MSVC v143 - VS 2022 C++ x64/x86 build tools
- Windows 10 (or 11) SDK (latest) - includes Universal CRT libs
- C++ CMake tools for Windows (optional but useful)
- Latest v143 Spectre-mitigated libs (optional)

## Steps
1. Run `Visual Studio Installer`.
2. Modify "Build Tools for Visual Studio 2022".
3. Select the workload above OR go to Individual Components and ensure the items are checked.
4. Apply changes.
5. Open a new Developer PowerShell or cmd (environment vars need refresh) and re-run build:

```
cargo build -p cngateway
```

## Verifying Installation
After install you should see (example path):
`C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\<version>\lib\x64\msvcrt.lib`

If still missing, ensure the Windows SDK path is on `LIB` env var. Running inside a "x64 Native Tools Command Prompt for VS 2022" sets this automatically.

## Temporary Workaround
If installing components right now is not possible, you can switch to the GNU toolchain (MinGW) temporarily (less recommended for production):
```
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
cargo build -p cngateway
```
Revert later to MSVC for better compatibility:
```
rustup default stable-x86_64-pc-windows-msvc
```

