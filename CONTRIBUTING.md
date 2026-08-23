# 🤝 Contributing Guidelines

Thank you for your interest in contributing to **Worktree & Workspace Companion**! Please follow these guidelines to keep the codebase maintainable, reliable, and secure.

---

## 1. Code & Commit Standards

- **Language**: All source code, docstrings, variable names, and comments MUST be in **English**.
- **Commits**: Use **Conventional Commits** in English (e.g. `feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`, `test: ...`).
- **Path Privacy**: NEVER include absolute local paths (e.g. `C:\Users\...`) in code, documentation, or commit messages. Always use relative paths or generic placeholders.
- **Security**: Never hardcode credentials, tokens, or sensitive data.

---

## 2. Development Workflow

### Prerequisites
- [Node.js](https://nodejs.org/) (v20+)
- [Rust & Cargo](https://rustup.rs/) (v1.80+)
- [Git](https://git-scm.com/) & [GitHub CLI](https://cli.github.com/)

### Setup & Commands (PowerShell)
```powershell
# 1. Install frontend dependencies
npm install

# 2. Run in desktop development mode (Hot Module Reloading)
npx tauri dev

# 3. Typecheck frontend
npm run check

# 4. Check Rust backend
cargo check --manifest-path src-tauri/Cargo.toml

# 5. Run backend tests
cd src-tauri; cargo test; cd ..
```

---

## 3. Pull Request Checklist

Before submitting a PR:
- [ ] Run `npm run check` to ensure Svelte and TypeScript have 0 errors and 0 warnings.
- [ ] Run `cargo check --manifest-path src-tauri/Cargo.toml` to verify backend syntax.
- [ ] Verify that destructive Git actions include pre-flight dirty checks.
- [ ] Ensure Windows subprocesses specify `CREATE_NO_WINDOW = 0x08000000`.
