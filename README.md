# 🚀 Worktree & Workspace Companion

[![Rust](https://img.shields.io/badge/Rust-1.97%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri&logoColor=white)](https://v2.tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-v5-ff3e00?logo=svelte&logoColor=white)](https://svelte.dev)
[![TailwindCSS](https://img.shields.io/badge/TailwindCSS-v3-38bdf8?logo=tailwindcss&logoColor=white)](https://tailwindcss.com)
[![GitHub CLI](https://img.shields.io/badge/GitHub%20CLI-Integrated-2088FF?logo=github&logoColor=white)](https://cli.github.com)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

*Read this in [English](#english) | Léelo en [Español](#español)*

---

<a name="english"></a>
## English

### 1. Project Description
**Worktree & Workspace Companion** is an ultra-lightweight, native system tray utility and Spotlight-style floating dashboard for Windows designed to streamline developer workflows with Git Worktrees and multi-account GitHub CLI setups.

#### ✨ Core Capabilities:
- 📊 **Active Worktrees Dashboard**: Discovers local repositories, parses active worktrees, detached heads, locked states, and dirty working trees with zero console flickering.
- 🧹 **Orphaned Worktree Cleaner**: Safely identifies worktrees whose remote upstream branch has been deleted or merged, equipped with pre-flight dirty checks to prevent accidental loss of uncommitted work.
- 🔄 **1-Click GitHub CLI Profile Switcher**: Instantly toggle between personal and corporate GitHub CLI identities (e.g. `gh auth switch`) and prevent author mismatch.
- 🪟 **Spotlight-style Floating Window**: Resides silently in the Windows system tray (<40 MB RAM in background), displays adjacent to the taskbar upon click, and auto-hides when losing focus.

---

### 2. Technologies Used
- **Backend**: Rust 1.97+, Tauri v2 (`tauri::tray::TrayIconBuilder`, `tauri-plugin-positioner`).
- **Frontend**: Svelte 5, TypeScript, Tailwind CSS, Lucide Icons.
- **Protocols & OS Integration**: Git Porcelain Protocol (`git worktree list --porcelain`), GitHub CLI (`gh`), Windows Win32 API (`CREATE_NO_WINDOW`).

---

### 3. Key Learnings
- **Subprocess Ergonomics in Windows**: Avoiding flashing terminal prompts when executing background developer CLIs requires low-level process creation flags (`CREATE_NO_WINDOW = 0x08000000`).
- **Git Porcelain Parsing**: Handling edge cases in `git worktree list --porcelain` (such as bare repositories lacking HEAD commits, locked worktrees, and detached states).
- **Safety Guardrails in Tooling**: Automating `git worktree remove` demands pre-flight dirty checks (`git status --porcelain`) before triggering destructive actions.

---

### 4. Local Setup Instructions

#### Prerequisites
- [Node.js](https://nodejs.org/) (v20+ recommended) & `npm`
- [Rust & Cargo](https://rustup.rs/) (v1.80+)
- [Git](https://git-scm.com/) & [GitHub CLI](https://cli.github.com/)

#### Installation & Development
```powershell
# 1. Clone the repository
git clone https://github.com/AnaCataVC/workspace-companion.git
cd workspace-companion

# 2. Install frontend dependencies
npm install

# 3. Run in desktop development mode (with Hot Module Reloading)
npx tauri dev

# 4. Run frontend standalone preview
npm run dev
```

#### Run Tests
```powershell
# Run backend Rust tests
cd src-tauri; cargo test; cd ..

# Check frontend types and build
npm run check
npm run build
```

---

### 5. Documentation & Architectural Decisions
- 📖 [Technical Architecture & Design](docs/architecture.md)
- 🏛️ [Architectural Decision Records (ADRs)](docs/adr/)
  - [ADR 0001: Tauri v2 System Tray & Window Positioning](docs/adr/0001-tauri-v2-system-tray-and-window-positioning.md)
  - [ADR 0002: Windows Subprocess Creation Flag](docs/adr/0002-windows-subprocess-no-window-flag.md)
  - [ADR 0003: Git Porcelain Protocol & Pre-flight Safety](docs/adr/0003-git-porcelain-preflight-safety.md)
- 🤝 [Contributing Guidelines](CONTRIBUTING.md)

---

<a name="español"></a>
## Español

### 1. Descripción del Proyecto
**Worktree & Workspace Companion** es una micro-herramienta nativa y ultra-ligera residente en la bandeja del sistema (*System Tray*) de Windows con ventana flotante estilo Spotlight, diseñada para potenciar la productividad gestionando Git Worktrees y múltiples cuentas de GitHub CLI.

#### ✨ Funcionalidades Principales:
- 📊 **Dashboard de Worktrees Activos**: Escanea repositorios locales y muestra worktrees, ramas, hashes de commit y estados sin parpadeos de consola.
- 🧹 **Limpiador Seguro de Worktrees Huérfanos**: Detecta ramas mergeadas o eliminadas remotamente con validación previa de cambios sin commitear para evitar pérdida accidental de código.
- 🔄 **Conmutador de Cuentas GitHub CLI en 1 Clic**: Alterna de forma inmediata entre cuentas de trabajo y personales (`gh auth switch`).
- 🪟 **Ventana Flotante Estilo Spotlight**: Permanece en la bandeja consumiendo menos de 40 MB de RAM, se abre al hacer clic sobre el tray y se auto-oculta al desenfocar (*auto-hide on blur*).

---

### 2. Tecnologías Utilizadas
- **Backend**: Rust 1.97+, Tauri v2 (`TrayIconBuilder`, `tauri-plugin-positioner`).
- **Frontend**: Svelte 5, TypeScript, Tailwind CSS, Lucide Icons.
- **Integración con el SO**: Protocolo Porcelain de Git, GitHub CLI (`gh`), Win32 `CREATE_NO_WINDOW`.

---

### 3. Aprendizajes Clave
- **Ergonomía de subprocesos en Windows**: Suprimir ventanas de consola emergentes al invocar `git` y `gh` en segundo plano mediante flags Win32 (`CREATE_NO_WINDOW`).
- **Robustez en el parseo de Git Porcelain**: Manejo de variantes como repositorios *bare*, worktrees bloqueados (*locked*) y estados *detached HEAD*.
- **Guardas de seguridad en herramientas destructivas**: Validación preventiva obligatoria (`git status --porcelain`) antes de ejecutar `git worktree remove`.

---

### 4. Instrucciones de Configuración Local

```powershell
# 1. Clonar el repositorio
git clone https://github.com/AnaCataVC/workspace-companion.git
cd workspace-companion

# 2. Instalar dependencias
npm install

# 3. Ejecutar en modo desarrollo con Tauri
npx tauri dev
```

---

### 5. Documentación y Decisiones Arquitectónicas
- 📖 [Arquitectura Técnica y Diseño](docs/architecture.md)
- 🏛️ [Registro de Decisiones Arquitectónicas (ADRs)](docs/adr/)
  - [ADR 0001: Integración con System Tray y Posicionamiento de Ventana](docs/adr/0001-tauri-v2-system-tray-and-window-positioning.md)
  - [ADR 0002: Flag Win32 de Creación de Subprocesos](docs/adr/0002-windows-subprocess-no-window-flag.md)
  - [ADR 0003: Protocolo Git Porcelain y Guardas de Seguridad](docs/adr/0003-git-porcelain-preflight-safety.md)
- 🤝 [Guía de Contribución](CONTRIBUTING.md)
