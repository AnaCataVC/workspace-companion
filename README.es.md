<p align="center">
  <img src="icon.png" alt="workspace-companion Logo" width="120" />
</p>

# Worktree & Workspace Companion

[English](README.md) | [Español](README.es.md)

[![Rust](https://img.shields.io/badge/Rust-1.97%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri&logoColor=white)](https://v2.tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-v5-ff3e00?logo=svelte&logoColor=white)](https://svelte.dev)
[![TailwindCSS](https://img.shields.io/badge/TailwindCSS-v3-38bdf8?logo=tailwindcss&logoColor=white)](https://tailwindcss.com)
[![GitHub CLI](https://img.shields.io/badge/GitHub%20CLI-Integrated-2088FF?logo=github&logoColor=white)](https://cli.github.com)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

---


### 1. Descripción del Proyecto
**Worktree & Workspace Companion** es una micro-herramienta nativa y ultra-ligera residente en la bandeja del sistema (*System Tray*) de Windows con ventana flotante estilo Spotlight, diseñada para potenciar la productividad gestionando Git Worktrees y múltiples cuentas de GitHub CLI.

#### ✨ Funcionalidades Principales:
- 📊 **Dashboard de Worktrees Activos**: Escanea repositorios locales y muestra worktrees, ramas, hashes de commit y estados sin parpadeos de consola.
- 💻 **Lanzadores Independientes de IDE y Terminal**: Lanzamiento configurable en 1 clic para editores de código (VS Code, Antigravity IDE, Cursor, Windsurf) y consolas (Windows Terminal, PowerShell, CMD, Git Bash, AGY CLI) con toggle opcional para mostrar u ocultar el botón de terminal.
- 🧹 **Limpiador Seguro de Worktrees Huérfanos**: Detecta ramas mergeadas o eliminadas remotamente con validación previa de cambios sin commitear para evitar pérdida accidental de código.
- 🌿 **Limpiador de Ramas**: Lista todas las ramas locales de los repos gestionados —incluidas las que no tienen worktree propio— marcadas como mergeadas, sin remoto o protegidas, para borrado seguro en lote. La rama por defecto y cualquier rama activa en un worktree nunca se pueden borrar, ni forzando una rama sin mergear.
- 🔄 **Conmutador de Cuentas GitHub CLI en 1 Clic**: Alterna de forma inmediata entre cuentas de trabajo y personales (`gh auth switch`).
- ⚡ **Acciones Rápidas y Limpieza Masiva (Quick Actions)**: Mantenimiento atómico del espacio de trabajo en 1 clic para un repositorio individual o todas las carpetas vigiladas a la vez. Ofrece cuatro preajustes: descartar cambios sin commitear protegiendo secretos en `.gitignore`, eliminar worktrees secundarios vinculados, limpiar ramas locales o ejecutar un **Total Fresh Start** en 5 fases hacia la rama por defecto. Protegido con auditoría previa de impacto (commits sin pushear) y confirmaciones armadas (cuenta regresiva de 3 s o confirmación por texto `"RESET"`).
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
- **Limpieza masiva atómica y aislamiento de bloqueos en Git**: Encadenar operaciones destructivas en múltiples repositorios exige una concurrencia estrictamente acotada: paralelizar entre repositorios independientes mediante Rayon (`par_iter`), manteniendo una ejecución estrictamente secuencial dentro de cada repositorio para evitar colisiones por `.git/index.lock` y bloqueos de archivos en Windows.

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
  - [ADR 0007: Arquitectura de Acciones Rápidas y Limpieza Masiva Atómica](docs/adr/0007-quick-actions-and-atomic-bulk-cleanup.md)
  - [Contrato Formal: Acciones Rápidas y Limpieza Masiva](docs/contracts/quick-actions-and-bulk-cleanup.contract.md)
- 🤝 [Guía de Contribución](CONTRIBUTING.md)

---

## Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo [LICENSE](LICENSE) para más detalles.

