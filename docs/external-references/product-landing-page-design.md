# Product Landing Page Design & Architecture

## Overview
This document outlines the design, architecture, and visual specifications for the **Worktree & Workspace Companion** product showcase & landing page.

## Key Architectural Decisions

1. **Location & Non-Interference with Tauri App**:
   - The desktop Tauri application uses root `/index.html` and Vite for desktop compilation.
   - The product landing page will be hosted in `website/index.html` (or `landing/index.html`) so it can be deployed independently to GitHub Pages / Vercel or previewed directly via browser (`file:///` protocol) without conflicting with the Vite Tauri build pipeline.

2. **Visual Design System & Aesthetics**:
   - **Theme**: Modern dark mode developer aesthetic (`#0f172a` slate / `#18181b` neutral, indigo/violet accent gradients, glassmorphism cards).
   - **Typography**: Clean modern sans-serif (`Inter`, system-ui font stack).
   - **App Logo**: Vector SVG branding incorporating the indigo rounded container and branch/worktree hierarchy iconography.
   - **Realistic Mockup**: A pixel-perfect Spotlight floating window representation displaying:
     - Header with app title, GitHub active profile tag (`AnaCataVC`), Quick actions.
     - Account filter tabs (`All`, `AnaCataVC`, `CataVillalobosC`).
     - Real worktree cards with status badges: `Active (Clean)`, `Working (2 modified files)`, `Orphaned (Safe to delete)`.
     - 1-click Editor action buttons (VS Code, Cursor, Explorer).
     - Branch switch indicator and commit SHA pill.

3. **Footer & Author Attribution**:
   - Prominent, elegant copy referencing author **Ana Catalina** (`AnaCataVC`) with direct links to GitHub portfolio (`https://github.com/AnaCataVC`).
   - Bilingual / English clean copy with MIT License notice.

4. **Performance & Standalone Portability**:
   - Pure modern HTML5 + CSS (with Tailwind styling / CSS variables) + Vanilla JS micro-interactions (e.g. interactive tab filtering or live mock tooltips).
   - Zero heavyweight runtime dependencies; instant loading on any browser.
