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
- 💻 **Independent IDE & Terminal Launchers**: Configurable 1-click launching for preferred code editors (VS Code, Antigravity IDE, Cursor, Windsurf) and consoles (Windows Terminal, PowerShell, CMD, Git Bash, AGY CLI) with optional terminal quick-button toggle.
- 🧹 **Orphaned Worktree Cleaner**: Safely identifies worktrees whose remote upstream branch has been deleted or merged, equipped with pre-flight dirty checks to prevent accidental loss of uncommitted work.
- 🌿 **Branch Cleaner**: Lists every local branch across managed repos — including ones with no worktree of their own — flagged as merged, remote-gone, or protected, for safe bulk deletion. The default branch and any checked-out branch are never deletable, even when forcing an unmerged one.
- 🔄 **1-Click GitHub CLI Profile Switcher**: Instantly toggle between personal and corporate GitHub CLI identities (e.g. `gh auth switch`) and prevent author mismatch.
- 🪟 **Desktop Window with System Tray Integration**: Resides silently in the Windows system tray (<40 MB RAM in background) and opens next to the tray icon.
  - **Toggle**: single-click the tray icon, or press **Alt+Space** from anywhere (global shortcut; if another app already owns it, the tray still works).
  - **Hide**: the window's **X** and **Esc** (with no dialog open) hide the panel to the tray instead of quitting; quit from the tray menu.
  - **Always on Top (Pin)**: the window stays visible as a standard desktop application without disappearing when moved or blurred; clicking the **Pin** button toggles *Always on Top* so it stays above other windows. The pin state is saved and restored on next launch.
- 🛡️ **Safer destructive actions**: discarding changes before a branch switch lists the files that will be lost and only arms the confirm button after a short delay; batch delete re-checks each selected worktree against the latest scan and flags selections hidden by the current filter.

#### 🧭 UI Anatomy at a Glance:
Each worktree row displays the **Git Branch Badge** (left interactive button to switch branches) and the **Physical Folder Path** (center/right name referenced by CLI tools and AI agents) side-by-side:
```text
┌────────────────────────────┐       ┌────────────────┐
│ [⎇] feature/branch-name [▼]│       │ worktree-folder│   [● dirty]   [🚀 IDE] [>_] [📁] [🗑]
└────────────────────────────┘       └────────────────┘
```
For a comprehensive breakdown of all toolbar buttons, status indicators, and view density modes, see the **[User Interface & Navigation Guide](docs/ui-guide.md)**.

---

### 2. Technologies Used
- **Backend**: Rust 1.97+, Tauri v2 (`tauri::tray::TrayIconBuilder`, `tauri-plugin-positioner`, `tauri-plugin-global-shortcut`).
- **Frontend**: Svelte 5, TypeScript, Tailwind CSS, Lucide Icons.
- **Protocols & OS Integration**: Git Porcelain Protocol (`git worktree list --porcelain`), GitHub CLI (`gh`), Windows Win32 API (`CREATE_NO_WINDOW`).

---

### 3. Key Learnings
- **Subprocess Ergonomics in Windows**: Avoiding flashing terminal prompts when executing background developer CLIs requires low-level process creation flags (`CREATE_NO_WINDOW = 0x08000000`).
- **IDE vs. CLI Disambiguation**: Differentiating between GUI desktop applications (e.g. `Antigravity.exe`) and command-line companion utilities (`agy.exe`) ensures predictable workspace launching.
- **Resilient Configuration Migration**: Using `#[serde(default)]` in Rust and nullish coalescing in TypeScript guarantees zero data loss and prevents deserialization crashes when extending configuration schemas.
- **Safety Guardrails in Tooling**: Automating `git worktree remove` demands pre-flight dirty checks (`git status --porcelain`) before triggering destructive actions.
- **Repo-Wide Context Caching & Delta-Patch Updates**: Computing orphan-detection facts (default branch, `branch -vv`, `branch --merged`) once per repository instead of once per worktree cuts subprocess spawns roughly 5x on multi-worktree repos; patching the in-memory worktree list after a single mutation — instead of re-scanning every watched folder — keeps the UI from flashing empty and avoids paying that cost for a one-worktree change.

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
- 🧭 [User Interface & Navigation Guide](docs/ui-guide.md)
- 📖 [Technical Architecture & Design](docs/architecture.md)
- 🏛️ [Architectural Decision Records (ADRs)](docs/adr/)
  - [ADR 0001: Tauri v2 System Tray & Window Positioning](docs/adr/0001-tauri-v2-system-tray-and-window-positioning.md)
  - [ADR 0002: Windows Subprocess Creation Flag](docs/adr/0002-windows-subprocess-no-window-flag.md)
  - [ADR 0003: Git Porcelain Protocol & Pre-flight Safety](docs/adr/0003-git-porcelain-preflight-safety.md)
  - [ADR 0004: Dual IDE & Terminal Separation and Subprocess Launcher Hardening](docs/adr/0004-ide-terminal-separation-and-subprocess-hardening.md)
  - [ADR 0005: Delta-Patch State Updates and Repo-Wide Context Caching Over Full Rescans](docs/adr/0005-delta-patch-state-and-repo-context-caching.md)
  - [ADR 0006: Branch Cleaner Safety Guards](docs/adr/0006-branch-cleaner-safety-guards.md)
- 🤝 [Contributing Guidelines](CONTRIBUTING.md)

---

<a name="español"></a>
## Español

### 1. Descripción del Proyecto
**Worktree & Workspace Companion** es una micro-herramienta nativa y ultra-ligera residente en la bandeja del sistema (*System Tray*) de Windows con ventana flotante estilo Spotlight, diseñada para potenciar la productividad gestionando Git Worktrees y múltiples cuentas de GitHub CLI.

#### ✨ Funcionalidades Principales:
- 📊 **Dashboard de Worktrees Activos**: Escanea repositorios locales y muestra worktrees, ramas, hashes de commit y estados sin parpadeos de consola.
- 💻 **Lanzadores Independientes de IDE y Terminal**: Lanzamiento configurable en 1 clic para editores de código (VS Code, Antigravity IDE, Cursor, Windsurf) y consolas (Windows Terminal, PowerShell, CMD, Git Bash, AGY CLI) con toggle opcional para mostrar u ocultar el botón de terminal.
- 🧹 **Limpiador Seguro de Worktrees Huérfanos**: Detecta ramas mergeadas o eliminadas remotamente con validación previa de cambios sin commitear para evitar pérdida accidental de código.
- 🌿 **Limpiador de Ramas**: Lista todas las ramas locales de los repos gestionados —incluidas las que no tienen worktree propio— marcadas como mergeadas, sin remoto o protegidas, para borrado seguro en lote. La rama por defecto y cualquier rama activa en un worktree nunca se pueden borrar, ni forzando una rama sin mergear.
- 🔄 **Conmutador de Cuentas GitHub CLI en 1 Clic**: Alterna de forma inmediata entre cuentas de trabajo y personales (`gh auth switch`).
- 🪟 **Ventana de Escritorio Integrada con el System Tray**: Permanece en la bandeja consumiendo menos de 40 MB de RAM y se abre junto al ícono del tray.
  - **Mostrar/ocultar**: un clic en el ícono del tray, o **Alt+Space** desde cualquier lugar (atajo global; si otra aplicación ya lo usa, el tray sigue funcionando).
  - **Ocultar**: la **X** de la ventana y **Esc** (sin ningún diálogo abierto) ocultan el panel en la bandeja en vez de cerrar la aplicación; para salir, usa el menú del tray.
  - **Siempre visible (Pin)**: la ventana permanece visible como una aplicación estándar sin desaparecer al moverla o cambiar de foco; el botón **Pin** alterna *Always on Top* para mantenerla encima de otras ventanas. El estado del pin se guarda y se restaura al volver a abrir la aplicación.
- 🛡️ **Acciones destructivas más seguras**: descartar cambios antes de cambiar de rama muestra los archivos que se perderán y solo habilita la confirmación tras una breve espera; el borrado en lote revalida cada worktree seleccionado contra el último escaneo y marca las selecciones ocultas por el filtro activo.

#### 🧭 Anatomía de la Interfaz:
Cada fila de worktree muestra en paralelo el **Badge de Rama Git** (botón interactivo para alternar ramas a la izquierda) y la **Carpeta Física** (nombre del directorio a la derecha referenciado por herramientas CLI y agentes de IA):
```text
┌────────────────────────────┐       ┌────────────────┐
│ [⎇] feature/nombre-rama [▼]│       │ carpeta-worktr │   [● dirty]   [🚀 IDE] [>_] [📁] [🗑]
└────────────────────────────┘       └────────────────┘
```
Para conocer el desglose detallado de todos los botones de la barra de herramientas, indicadores de estado y modos de vista, consulta la **[Guía de Interfaz de Usuario y Navegación](docs/ui-guide.md)**.

---

### 2. Tecnologías Utilizadas
- **Backend**: Rust 1.97+, Tauri v2 (`TrayIconBuilder`, `tauri-plugin-positioner`, `tauri-plugin-global-shortcut`).
- **Frontend**: Svelte 5, TypeScript, Tailwind CSS, Lucide Icons.
- **Integración con el SO**: Protocolo Porcelain de Git, GitHub CLI (`gh`), Win32 `CREATE_NO_WINDOW`.

---

### 3. Aprendizajes Clave
- **Ergonomía de subprocesos en Windows**: Suprimir ventanas de consola emergentes al invocar `git` y `gh` en segundo plano mediante flags Win32 (`CREATE_NO_WINDOW`).
- **Disambiguación entre IDE y CLI**: Diferenciar entre aplicaciones GUI de escritorio (ej. `Antigravity.exe`) y utilidades CLI auxiliares (`agy.exe`) para un lanzamiento predecible del espacio de trabajo.
- **Migración resiliente de configuración**: Uso de `#[serde(default)]` en Rust y nullish coalescing en TypeScript para garantizar compatibilidad hacia atrás total y prevenir fallos de deserialización al extender el esquema de configuración.
- **Guardas de seguridad en herramientas destructivas**: Validación preventiva obligatoria (`git status --porcelain`) antes de ejecutar `git worktree remove`.
- **Caché de contexto por repositorio y actualizaciones parciales**: Calcular los datos de detección de huérfanos (rama por defecto, `branch -vv`, `branch --merged`) una sola vez por repositorio, en vez de una vez por worktree, reduce ~5x los subprocesos lanzados en repos con varios worktrees; parchar la lista de worktrees en memoria tras una sola mutación —en vez de re-escanear todas las carpetas vigiladas— evita que la interfaz se vea vacía por un instante y evita pagar ese costo por el cambio de un solo worktree.

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
- 🧭 [Guía de Interfaz de Usuario y Navegación](docs/ui-guide.md)
- 📖 [Arquitectura Técnica y Diseño](docs/architecture.md)
- 🏛️ [Registro de Decisiones Arquitectónicas (ADRs)](docs/adr/)
  - [ADR 0001: Integración con System Tray y Posicionamiento de Ventana](docs/adr/0001-tauri-v2-system-tray-and-window-positioning.md)
  - [ADR 0002: Flag Win32 de Creación de Subprocesos](docs/adr/0002-windows-subprocess-no-window-flag.md)
  - [ADR 0003: Protocolo Git Porcelain y Guardas de Seguridad](docs/adr/0003-git-porcelain-preflight-safety.md)
  - [ADR 0004: Separación Dual de IDE y Terminal y Endurecimiento de Subprocesos](docs/adr/0004-ide-terminal-separation-and-subprocess-hardening.md)
  - [ADR 0005: Actualizaciones Parciales de Estado y Caché de Contexto por Repositorio](docs/adr/0005-delta-patch-state-and-repo-context-caching.md)
  - [ADR 0006: Guardas de Seguridad del Limpiador de Ramas](docs/adr/0006-branch-cleaner-safety-guards.md)
- 🤝 [Guía de Contribución](CONTRIBUTING.md)
