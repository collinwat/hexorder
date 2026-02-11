# Hex & Counter Wargame Research

## Initial Prompt

> I want to first source a hex grid military game system that exists in the open. I want it to serve as our first example that the application should support. Can you research what hex and counter board games exist, where their assets already exist that we can use, and the rules are available to use to read and implement.

---

## Complete Research

### TIER 1: Best Reference Game Candidates

#### 1. Battle for Moscow (Frank Chadwick, 1986)

**The strongest single candidate as a reference game.**

- **Designer:** Frank Chadwick (GDW)
- **Period:** WWII Eastern Front, Operation Typhoon 1941
- **Complexity:** Simple/Introductory (designed explicitly to introduce new players to hex-and-counter wargaming)
- **Components:** ~39 counters (Soviet armies + German corps, each with full/half-strength sides), 1 hex map, CRT, Terrain Effects Chart, Turn Record Track
- **Rules:** ~4 pages of rules, 7 turns per game
- **Game Mechanics:** All core hex wargame mechanics present:
  - Hex grid movement with movement points
  - Zones of Control (rigid)
  - Odds-ratio Combat Results Table (CRT)
  - Terrain effects on combat and movement (forests, cities, fortifications, rivers, railroads)
  - Step losses (full-strength to half-strength)
  - Replacements/reinforcements
  - Supply rules (simplified)
- **License/Availability:** Frank Chadwick made the game freely available for personal/recreational use after GDW closed in 1996. The copyright remains with Chadwick. **NOT open source** -- it is free to download and print, but "not for commercial or mass reproduction."
- **URLs:**
  - Rules & components: https://grognard.com/bfm/game.html
  - Alternative site: https://oberlabs.com/b4m/rules.html
  - Rules PDF: https://tesera.ru/images/items/160567/B4M_rules_booklet.pdf
  - Internet Archive: https://archive.org/details/bmoscow
- **Why it works:** Perfect complexity level (~39 counters, single map, 4 pages of rules), represents all core hex wargame mechanics, designed specifically as an intro game. Very well documented.
- **Why it might not work:** Copyright is retained by Chadwick. Free for personal use but not formally open-licensed. Cannot redistribute modified versions or include in an open-source project without permission.

#### 2. Napoleon at Waterloo (SPI, James Dunnigan, 1971)

**The original introductory hex wargame.**

- **Designer:** James F. Dunnigan
- **Period:** Napoleonic Wars, Battle of Waterloo 1815
- **Complexity:** Very simple (was given away free to introduce people to wargaming)
- **Components:** 61 counters (cavalry and infantry divisions), 1 small hex map (11"x13"), CRT
- **Rules:** ~6 pages (4 sides of A4)
- **Game Mechanics:**
  - I-Go-You-Go turn structure
  - Simple hex movement
  - Odds-ratio CRT
  - Terrain effects
  - Zones of control
- **License/Availability:** SPI is defunct. The rules, map, and counters are available for download at spigames.net, which provides resources "to help people play and enjoy the old SPI games they own." The legal status is ambiguous -- Decision Games later acquired SPI rights and republished a revised edition in 2014.
- **URLs:**
  - Rules PDF: https://www.spigames.net/PDFv2/NapoleonatWaterloo.pdf
  - Print and Play: https://www.kobudovenlo.nl/napoleonatwaterloo/Napoleon%20at%20Waterloo%20PnP.htm
  - SPI Downloads page: https://www.spigames.net/rules_downloads.htm
  - Steam Workshop (Tabletop Simulator): https://steamcommunity.com/sharedfiles/filedetails/?id=340557668
- **Why it works:** Even simpler than Battle for Moscow. Historically important as the original introductory wargame. Complete PnP files available.
- **Why it might not work:** Legally ambiguous -- Decision Games may hold rights. Not formally open-licensed. The game is almost too simple (limited terrain variety).

#### 3. Ogre (Steve Jackson Games, 1977)

**Simple, well-documented, rules freely published by the designer.**

- **Designer:** Steve Jackson
- **Period:** Science fiction (near-future armored warfare)
- **Complexity:** Simple
- **Components:** ~140 counters (Pocket Edition), 1 hex map, simple CRT
- **Rules:** ~16 pages (Pocket Edition rulebook)
- **Game Mechanics:**
  - Hex grid movement
  - Asymmetric gameplay (one mega-tank vs. many smaller units)
  - Simple attack/defense ratio combat
  - Unit damage tracking
- **License/Availability:** Rules PDFs are hosted on Steve Jackson Games' official website. The rules are freely downloadable, but SJG retains full copyright. The game is still actively sold.
- **URLs:**
  - Rulebook PDF: https://www.sjgames.com/ogre/kickstarter/ogre_rulebook.pdf
  - Ogre + G.E.V. rules: https://www.sjgames.com/ogre/products/ogregev/img/ogre-rules-new.pdf
  - Pocket Edition rules: https://tesera.ru/images/items/155413/Pocket_Ogre_Rules_6-14-12.pdf
  - Official site: https://www.sjgames.com/ogre/
- **Why it works:** Extremely well-documented rules. Iconic game. Simple enough for first implementation. Rules freely available online.
- **Why it might not work:** Sci-fi rather than historical. Actively commercial product with strong copyright. Asymmetric design is not typical of standard hex wargames (one side has a single super-unit). Cannot use the game's assets.

#### 4. Hex Encounters (Evan D'Alessandro, 2024)

**Purpose-built "simplest possible" hex-and-counter teaching game.**

- **Designer:** Evan D'Alessandro (PhD student, King's College London, War Studies)
- **Period:** Modern brigade combat (generic/unspecified specific battle)
- **Complexity:** Deliberately minimal
- **Components:** Counters (20x10mm blocks), 2-page map, CRT, Terrain Effects Chart
- **Game Mechanics:**
  - All standard hex wargame features present
  - Movement with movement points
  - Zones of control
  - CRT with column shifts
  - Terrain effects on movement and combat
  - Stacking limits
  - Units with Movement-Combat stats (or Movement-Offense-Defense)
- **License/Availability:** Copyright 2024 Evan D'Alessandro. No explicit open license found. Free to download.
- **URLs:**
  - Game page: https://evandalessandro.com/hex-encounters/
  - Rules PDF (v4): https://evandalessandro.com/wp-content/uploads/2024/02/hexencountersrulesv4.pdf
- **Why it works:** Explicitly designed to be the "simplest possible game that includes all standard features of a hex and counter wargame." Perfect for validating a tool. Includes digital (Google Slides) version.
- **Why it might not work:** Copyright retained, no open license. Generic scenario rather than historical. Less community recognition.

---

### TIER 2: Open Source Digital Implementations

#### 5. VASSAL Hex-and-Counters Template (jzedwards)

**CC0-licensed template for building hex wargames digitally.**

- **License:** CC0 (Creative Commons Zero -- Public Domain)
- **What it is:** A VASSAL module template providing all the infrastructure for a hex-and-counter wargame
- **Includes:**
  - Hex grid system with configurable hex sizes
  - NATO unit symbol templates (infantry, mechanized, anti-aircraft)
  - Counter rotation (6 directions), movement marking, step-loss tracking
  - CRT chart window, terrain effects chart, turn track
  - Dice rolling (d6, 2d6)
  - Eliminated units tracking ("dead pile")
  - Multiple counter color variants and prototype definitions
- **URL:** https://github.com/jzedwards/vassal-hex-n-counters-template
- **Why it works:** CC0 licensed (completely free to use for anything). Provides a reusable template with NATO symbology and all standard wargame infrastructure. Could serve as a reference for what a hex-and-counter wargame system needs.
- **Why it might not work:** It is a VASSAL module (Java/XML), not a standalone game engine. No specific game rules included -- just the framework.

#### 6. Wargame LaTeX Package (Christian Holm Christensen)

**CC-BY-SA 4.0 licensed system for creating complete hex wargames.**

- **License:** Creative Commons Attribution-ShareAlike 4.0 International (CC-BY-SA-4.0)
- **What it is:** A LaTeX package that generates complete print-and-play wargames (rules, hex maps, counter sheets, charts) plus exports to VASSAL modules
- **Features:**
  - Hex map generation with terrain types
  - Counter/chit creation with NATO APP-6 symbology
  - Counter sheet layout
  - Order of Battle charts
  - CRT and other chart generation
  - VASSAL module export via `wgexport.py`
- **Example games included:** Tannenberg (introductory), plus tutorial game with clear, woods, and mountain terrain
- **URLs:**
  - GitLab: https://gitlab.com/wargames_tex/wargame_tex
  - CTAN package page: https://ctan.org/pkg/wargame
  - Tutorial PDF: https://ctan.math.illinois.edu/macros/latex/contrib/wargame/doc/tutorial/game.pdf
  - Documentation: https://ctan.math.illinois.edu/macros/latex/contrib/wargame/doc/wargame.pdf
- **Why it works:** Fully open source, CC-BY-SA licensed. Includes complete example games. Defines a formal data model for hex wargames (terrain, counters, OOB, charts). Excellent reference for what a wargame design tool needs to produce.
- **Why it might not work:** LaTeX-based (not a game engine). Output is PDF, not interactive. CC-BY-SA requires attribution and share-alike.

#### 7. Hex-Wargame-JavaScript (yiyuezhuo)

**MIT-licensed browser-based hex wargame with scenario editor.**

- **License:** MIT
- **What it is:** A playable hex wargame in JavaScript with a web-based scenario editor
- **Features:**
  - Hex maps with terrain types defined via CSV
  - NATO military symbology for units
  - Combat Results Table
  - Scenario editor (under development)
  - Terrain system inspired by HPS & John Tiller games
- **URL:** https://github.com/yiyuezhuo/Hex-Wargame-JavaScript
- **Why it works:** MIT licensed, includes scenario editor, uses CSV for data (easy to parse), directly represents a hex wargame system.
- **Why it might not work:** JavaScript/DOM-based (not Bevy). 23 commits, appears to be a small personal project. Limited documentation.

#### 8. Crimson Fields

**GPL-licensed hex tactical wargame with map editor.**

- **License:** GNU GPL v2
- **What it is:** A complete turn-based tactical war game on a hex grid with custom map and campaign creation tools
- **Features:**
  - Hex grid with multiple terrain types
  - Unit types with customizable stats
  - Built-in map/campaign editor
  - Cross-platform (Windows, Linux, Mac)
- **URLs:**
  - LibreGameWiki: https://libregamewiki.org/Crimson_Fields
  - Source: https://gitlab.com/osgames/crimson.git
- **Why it works:** Complete, working hex wargame with editor. GPL licensed. Modifiable unit and terrain data.
- **Why it might not work:** Last release 0.5.3 was in 2009 -- project appears dormant. C++ codebase. Not a historical wargame (generic/fictional scenarios). GPL v2 is copyleft.

#### 9. Warfare-RS (mjhouse)

**GPL-3.0 hex wargame in Rust/Bevy.**

- **License:** GPL-3.0
- **What it is:** A multiplayer 2D turn-based strategy wargame written in Rust using Bevy
- **Features:**
  - Procedural hex map generation (up to 1000x1000 tiles)
  - Environmental simulation (biome, elevation, temperature)
  - Unit creation with individual soldier tracking
  - Basic networking
- **URL:** https://github.com/mjhouse/warfare-rs
- **Why it works:** Same technology stack (Rust + Bevy). Hex grid implementation already exists. Open source.
- **Why it might not work:** Early development (76 commits). No formal wargame rules system. Modern setting rather than historical. Focused on simulation rather than board-game-style mechanics. GPL-3.0 copyleft.

---

### TIER 3: Open Asset Libraries

#### 10. Kenney Hexagon Pack (CC0)

- **License:** CC0 (Public Domain)
- **Contents:** 310+ hex tiles across themes: medieval, military, lumbermill, modern, sci-fi, western. Over 80 pre-made tiles plus buildings, objects, and details. Includes separate PNGs, spritesheets, and vector source files.
- **URLs:**
  - https://kenney.nl/assets/hexagon-pack
  - OpenGameArt mirror: https://opengameart.org/content/hexagon-pack-310x
  - GitHub mirror: https://github.com/utgarda/kenney-hexagon
- **Additional Kenney hex packs:**
  - Hexagon Tiles (90 assets): https://kenney.nl/assets/hexagon-tiles
  - Hexagon Kit (70 assets): https://kenney.nl/assets/hexagon-kit

#### 11. OpenGameArt Hex Tileset Pack (CC0)

- **License:** CC0
- **Contents:** Terrain tiles, roads, units, mountains, bridges, buildings at 32x32 pixels with spritesheet
- **URL:** https://opengameart.org/content/hex-tileset-pack

#### 12. Hexset v0.1.1 Pixel Art Terrain (CC0)

- **License:** CC0
- **Contents:** Pixel art hex terrain tileset with 3 variants per tile type, in both "cartoonish" and "subtle" styles
- **URL:** https://opengameart.org/content/hexset-v011-hex-pixel-art-terrain-tileset

#### 13. 180+ Seamless Hex Tiles

- **Contents:** 186 realistic hex tiles in Desert, Volcanic, Forest, Lava, Grassy, Dirt, Water, Rocky terrain types. Thick and Flat tile variants with 7 outline styles.
- **URL:** https://opengameart.org/content/180-seamless-hex-tiles

#### 14. Milsymbol -- NATO Military Symbols (MIT)

- **License:** MIT
- **What it is:** Pure JavaScript library generating MIL-STD-2525 and STANAG APP-6 military unit symbols as SVG
- **Features:** Generates SVG military symbols programmatically. Customizable fill, frame, color, size, stroke. Supports infantry, armor, artillery, and hundreds of other unit types. Generates 1000 symbols in <20ms.
- **URL:** https://github.com/spatialillusions/milsymbol
- **Directly useful for:** Generating NATO-standard counter art for any wargame

#### 15. Military-Symbol Python Module

- **What it is:** Python module generating NATO APP-6(E) compliant military symbols as SVG
- **Input:** SIDC codes or natural-language names (e.g., "friendly infantry platoon")
- **URL:** https://pypi.org/project/military-symbol/

#### 16. Generic Wargame Maps (Public Domain)

- **License:** Public Domain
- **Contents:** GIF and XCF hex map templates sized for 1/2" gaming counters
- **URL:** http://ludo.iwarp.com/gwm/

---

### TIER 4: Design & Creation Tools

#### 17. WarGame Counter Generator (Apache-2.0)

- **License:** Apache-2.0
- **What it is:** Python tool using Cairo to generate wargame counters
- **URL:** https://github.com/KordianChi/WarGame_Counter_Generator

#### 18. Countersheets Inkscape Extension (GPL-2.0+)

- **License:** GPL-2.0+
- **What it is:** Inkscape extension for laying out sheets of counters/cards/tiles from CSV data
- **Features:** Two-sided counters, CSV-driven data merge, text and image replacement
- **URL:** https://github.com/lifelike/countersheetsextension

#### 19. One-Page Wargame (Browser-based template)

- **What it is:** Simple hex wargame framework using only HTML/CSS/JS
- **Features:** Hex grid generation, draggable unit counters, customizable terrain
- **URL:** https://github.com/GiantRavens/onepage-wargame

#### 20. Hex Map Maker (Web tool)

- **What it is:** Web-based hex map creator that exports to editable SVG
- **URL:** https://molotovcockatiel.com/hex-map-maker/

---

### TIER 5: Important Community Resources

#### 21. SPI Games Archive (spigames.net)

- **What it is:** Hundreds of classic SPI wargame rules, maps, and counters available for download
- **Games with complete downloads (rules + map + counters):** Dozens of classic games
- **URL:** https://www.spigames.net/rules_downloads.htm
- **Note:** Legal status ambiguous. Site says "resource to help people play and enjoy the old SPI games they own."

#### 22. BGG Free Print-and-Play Wargames Lists

- Metalist: https://boardgamegeek.com/geeklist/45253/best-free-print-play-wargames-metalist
- Free Historical PnP Wargames: https://boardgamegeek.com/geeklist/23887/free-print-and-play-historical-wargames
- Simplest Hex-and-Counter: https://boardgamegeek.com/thread/2049232/simplest-possible-hex-and-counter-wargame
- Entry Level Hex PnP: https://boardgamegeek.com/thread/1493875/entry-level-hex-and-counter-pnp

#### 23. VASSAL Module Library

- **What it is:** 3200+ board game modules, many hex wargames
- **License:** VASSAL engine is LGPL; individual modules vary
- **URL:** https://vassalengine.org/library
- **Note:** Modules contain digitized maps and counters. Usage rights depend on publisher permissions.

#### 24. Grognard.com Wargame Resources

- **What it is:** Huge archive of wargame extras, variants, upgraded components
- **URL:** https://grognard.com/Board.aspx

---

## Summary

There is no single game that perfectly combines open rules, open assets, AND a complete representative hex-and-counter wargame. But there are strong candidates and a clear path forward.

### Top Game Candidates

| Game | Period | Scale | Rules | License | Best For |
|------|--------|-------|-------|---------|----------|
| **Battle for Moscow** | WWII Eastern Front | ~39 counters, 1 map, 7 turns | ~4 pages | Free personal use, copyright retained | Perfect complexity, all core mechanics |
| **Napoleon at Waterloo** | Napoleonic, Waterloo | 61 counters, small map | ~6 pages | Legally ambiguous (SPI defunct) | Historically important, very simple |
| **Hex Encounters** | Modern brigade (generic) | Minimal | ~2 pages | Copyright retained, free download | "Simplest possible" hex wargame |
| **Ogre** | Sci-fi | ~140 counters | ~16 pages | Copyright retained, free download | Well-documented but not historical |

### Open-License Assets Available

| Asset | License | What |
|-------|---------|------|
| **Kenney Hexagon Pack** | CC0 (public domain) | 310+ hex tiles including military theme |
| **OpenGameArt Hex Tileset** | CC0 | Terrain tiles, roads, units, mountains |
| **Milsymbol** | MIT | Generates NATO MIL-STD-2525 military symbols as SVG |
| **VASSAL Hex Template** | CC0 | Complete hex wargame infrastructure template |
| **Wargame LaTeX Package** | CC-BY-SA 4.0 | Full wargame generation system with example games (Tannenberg) |

### Open-Source Codebases

| Project | License | Stack | Notes |
|---------|---------|-------|-------|
| **warfare-rs** | GPL-3.0 | Rust/Bevy | Hex wargame, same tech stack, early development |
| **Crimson Fields** | GPL-2.0 | C++ | Complete hex tactical game with map editor, dormant since 2009 |
| **Hex-Wargame-JS** | MIT | JavaScript | Hex wargame with scenario editor |

---

## Recommendations

### Key Legal Insight

**Game mechanics are not copyrightable.** The odds-ratio CRT, zones of control, hex movement, and terrain effects are shared conventions of the genre. Only specific creative expression (art, rules text, scenario narrative) is protected. The tool can implement all standard hex wargame mechanics while using original assets and scenarios.

### Recommended Approach

1. **Study Battle for Moscow's mechanics** as the reference design -- it has the right complexity level and covers every core hex wargame mechanic (movement points, ZOC, odds-ratio CRT, terrain effects, step losses, reinforcements)
2. **Use CC0/MIT assets** for visuals -- Kenney hex tiles (CC0) + Milsymbol NATO symbols (MIT)
3. **Use the Wargame LaTeX package** (CC-BY-SA 4.0) as a reference for data modeling -- it defines a formal structure for terrain, counters, orders of battle, and charts
4. **Use the VASSAL CC0 template** as a reference for required UI/infrastructure features
5. **Create an original scenario** that exercises all the key mechanics with no licensing concerns

### Core Mechanics to Implement (from Battle for Moscow)

These are the standard hex wargame mechanics that the tool must support:

- **Hex grid** with axial coordinates
- **Terrain types**: Clear, Forest, City, Fortification, River, Railroad
- **Unit counters** with Attack-Defense-Movement values and full/half strength sides
- **Movement** with movement point costs per terrain type
- **Zones of Control** (rigid -- must stop when entering enemy ZOC)
- **Combat** via odds-ratio Combat Results Table (attacker strength / defender strength)
- **Terrain effects** on both combat and movement
- **Step losses** (full-strength to half-strength to eliminated)
- **Reinforcements and replacements** on specific turns
- **Turn structure**: Move phase, Combat phase, alternating sides
- **Victory conditions**: Control of specific hexes or elimination thresholds
