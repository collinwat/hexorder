# Hexorder Brand Identity

## Name

**Hexorder** — a compound of _hex_ (the grid) and _order_ (rules, structure, military orders).

## Purpose

A game system design tool for historical military tabletop wargames. It is a craftsman's workbench,
not a consumer product. The brand should feel like opening a war room — serious, precise,
authoritative.

## Visual Feel: Dark Forge / Workshop

- Dark backgrounds with minimal chrome
- A single hot accent color draws attention to interactive elements
- The tool recedes; the user's work (the hex board, the game system) is the focus
- No decoration for its own sake — every visual element earns its place

## Inspiration

1800s military insignia, particularly the **Union Army corps badge system** (1861-65). These badges
were bold geometric shapes — circles, trefoils, diamonds, Maltese crosses, stars — assigned to corps
and colored by division (red/white/blue). They were designed for instant recognition at distance on
a chaotic battlefield. Hexorder adapts this tradition: a hexagonal badge that is both a grid element
and a military mark.

## Color Palette

### Primary

| Role                 | Color      | Hex       | Usage                                           |
| -------------------- | ---------- | --------- | ----------------------------------------------- |
| Background (deep)    | Near-black | `#0a0a0a` | Deepest UI panels                               |
| Background (panel)   | Dark gray  | `#191919` | Panel fill                                      |
| Background (surface) | Charcoal   | `#232323` | Interactive surface areas                       |
| Accent (primary)     | Teal       | `#005c80` | Selection highlights, active states, brand mark |
| Accent (warm)        | Amber/gold | `#c89640` | Secondary accent, emphasis, warmth              |

### Supporting

| Role             | Color       | Hex       | Usage                    |
| ---------------- | ----------- | --------- | ------------------------ |
| Text (primary)   | Off-white   | `#e0e0e0` | Body text                |
| Text (secondary) | Medium gray | `#808080` | Labels, secondary info   |
| Text (disabled)  | Dark gray   | `#505050` | Inactive elements        |
| Border (subtle)  | Dark gray   | `#3c3c3c` | Panel borders, dividers  |
| Danger           | Muted red   | `#c85050` | Delete actions, warnings |
| Success          | Muted green | `#509850` | Valid states, success    |

### Cell Type Defaults (naturalistic)

These are starter colors for the default game system. Users will define their own.

| Cell Type | Color       | Hex       |
| --------- | ----------- | --------- |
| Plains    | Sage green  | `#99cc66` |
| Forest    | Deep green  | `#338033` |
| Water     | Medium blue | `#3366cc` |
| Mountain  | Warm brown  | `#80664d` |
| Road      | Tan         | `#b39966` |

## Icon

### Concept

A hexagonal badge inspired by 1800s military corps insignia. The hexagon is both the fundamental
grid unit and the badge shape — form follows function.

### Design

- **Shape**: Regular hexagon (pointy-top, matching the grid orientation)
- **Interior motif**: A six-rayed star or compass mark formed by connecting alternate vertices —
  evokes both military insignia and the six cardinal directions of hex geometry
- **Color**: Teal accent on dark background, with amber highlight on the central mark
- **Style**: Bold, geometric, high contrast — readable at 16x16 pixels

### Sizes needed (macOS)

- 16x16, 32x32, 64x64, 128x128, 256x256, 512x512, 1024x1024
- Format: `.icns` (macOS app bundle) or `.iconset` folder with PNGs

## UI Implementation Notes

This document is the **single source of truth** for all UI colors. When adding or modifying editor
UI code, pull values from the palette above — do not introduce ad-hoc colors.

### Current alignment (as of M2)

The editor dark theme in `src/editor_ui/systems.rs` (`configure_theme`) already uses the brand
palette for backgrounds, teal accent, borders, and danger. The egui dark defaults handle text colors
acceptably.

### Not yet introduced

- **Amber/gold accent** (`#c89640`) — present in the icon but not yet in the UI. Introduce it
  incrementally in future milestones for:
    - Section headings or active tab labels
    - Selected cell type border in the palette (currently white)
    - Game System name/version display
    - Primary action buttons (Create, Save)
- **Text colors** (`#e0e0e0`, `#808080`, `#505050`) — currently relying on egui dark defaults.
  Explicitly set these when the defaults diverge from the brand or when finer control is needed.

### Rules for new UI work

1. Use brand palette values, not arbitrary grays or colors
2. Teal (`#005c80`) = interactive state (selection, hover accents, links)
3. Amber (`#c89640`) = emphasis and warmth (headings, primary actions, active indicators)
4. Reserve danger red (`#c85050`) strictly for destructive actions
5. When in doubt, keep it dark and let the user's content (the hex board) be the brightest thing on
   screen

## Typography

- System fonts only (no custom fonts shipped)
- Monospace for data values and coordinates
- Sans-serif for UI labels and headings

## Voice

- Direct, concise, technical
- Military clarity: say what you mean, no filler
- Tool language: "define", "configure", "inspect", "deploy" — not "play", "enjoy", "discover"
