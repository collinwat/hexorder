# Exhaustive Survey of Hex-and-Counter Wargame Mechanics

## Purpose

This document catalogs the full design space of hex-and-counter wargame mechanics for the purpose of
designing a game system definition tool (HexOrder) that must support defining arbitrary hex wargame
rule sets. Each mechanic is described with its function, prevalence, example games, and implications
for the tool's data model.

---

## Area 1: Core Universal Mechanics

These appear in nearly every hex-and-counter wargame and form the irreducible foundation.

---

### 1.1 Hex Grid Systems

**How it works:** The game map is overlaid with a hexagonal grid. Each hex represents a geographic
area at a specific scale (e.g., 1 hex = 5 miles for operational, 200 meters for tactical). Hexes
provide six equidistant neighbors, enabling more natural movement and facing than square grids.

**Coordinate Systems:**

| System                  | Description                                                                                    | Pros                                                                                        | Cons                                                                        |
| ----------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| **Offset**              | Staggered rows/columns; odd/even rows shift. Traditional board game labeling (e.g., hex 1215). | Intuitive for printed maps; matches physical counter placement                              | Algorithms for distance/neighbors are irregular; odd/even row parity issues |
| **Axial (Trapezoidal)** | Two coordinates (q, r) on skewed axes. Third coordinate s = -q-r is implicit.                  | Simple storage; most algorithms work cleanly; easy conversion to/from cube                  | Less intuitive for humans; negative coordinates common                      |
| **Cube**                | Three coordinates (x, y, z) where x+y+z=0. Each hex sits on a plane in 3D space.               | Simplest algorithms: distance = max(abs(dx), abs(dy), abs(dz)); rotation trivial; symmetric | Redundant storage (3 values for 2D position); constraint must be maintained |

**Commonality:** Universal.

**Example games:** Every hex wargame uses one of these. Board games use offset labeling (e.g., "hex
2413"). Digital implementations favor axial/cube internally.

**Data model implications:**

- Must support all three coordinate systems with conversion between them.
- Hex identity is the fundamental spatial key -- everything (terrain, units, ownership, control) is
  indexed by hex.
- Map must support hex-to-hex adjacency queries, distance calculations, pathfinding, line-of-sight
  ray tracing, and ring/cone/wedge area selections.
- Must support variable hex orientations: flat-top vs. pointy-top hexes.
- Must support hex labeling schemes (row-column numbering) independent of internal coordinate
  system.

**Sources:** [Red Blob Games: Hexagonal Grids](https://www.redblobgames.com/grids/hexagons/),
[Hex Grids and Cube Coordinates](https://backdrifting.net/post/064_hex_grids)

---

### 1.2 Movement Systems

**How it works:** Each unit has a Movement Allowance (MA) representing movement points (MPs)
available per turn. Each hex costs a certain number of MPs to enter, determined by the terrain in
the hex and the unit type. A unit moves hex-by-hex, deducting terrain costs, until it runs out of
MPs or chooses to stop.

**Key sub-mechanics:**

| Sub-mechanic                   | Description                                                                                                        | Commonality           |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------ | --------------------- |
| **Movement Points (MPs)**      | Numeric budget per unit per phase. Terrain costs deducted per hex entered.                                         | Universal             |
| **Terrain costs by unit type** | Infantry pays 1 MP for clear, 2 for woods; vehicles pay 1 for clear, 3 for woods, prohibited in swamp.             | Universal             |
| **Road movement**              | Entering a hex via road hexside costs only the road rate, regardless of terrain in the hex.                        | Universal             |
| **Minimum move**               | A unit can always move at least one hex, even if the terrain cost exceeds its MA.                                  | Very common           |
| **Strategic movement**         | Enhanced movement rate (often doubled MA) in exchange for restrictions (no combat, no enemy ZOC entry, road-only). | Common                |
| **Rail movement**              | Units move along connected rail hexes at very high rates, limited by rail capacity points per turn.                | Common (operational+) |
| **Sea/naval movement**         | Units move between ports or along sea hexes using naval transport points.                                          | Common (theater+)     |
| **Hexside costs**              | Some terrain is on hex edges (rivers, ridges, walls). Crossing costs additional MPs.                               | Very common           |

**Example games:** Panzerblitz (1970) codified MP-based movement. Every SPI, Avalon Hill, and GMT
game uses variants. OCS adds fuel costs for mechanized movement.

**Data model implications:**

- Units need: movement_allowance, movement_type (foot, tracked, wheeled, horse, etc.)
- Terrain Effects Chart (TEC) is a matrix: terrain_type x unit_movement_type -> cost_in_MPs.
- Hexside terrain is distinct from hex terrain; both must be modeled.
- Roads override hex terrain cost when entering via the road hexside.
- Multiple movement modes per unit (normal, strategic, exploitation) with different MA values and
  restrictions.
- Rail/sea movement needs a transport capacity system separate from MP movement.

**Sources:**
[Movement Points: Standard Combat System](https://chrisbaer.net/mp/2008/02/06/movement-points-standard-combat-system/),
[Rules for Wargames](https://www.cs.hmc.edu/~dhamm/wargames/rules.html)

---

### 1.3 Combat Resolution Systems

**How it works:** The attacking player designates one or more units to attack an enemy-occupied hex.
Combat strengths are compared, modifiers applied, dice rolled, and results read from a Combat
Results Table (CRT) or computed via a formula.

**CRT Types:**

| Type                     | Calculation                                                             | Description                                                                     | Example Games                                                            |
| ------------------------ | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| **Odds-Ratio CRT**       | Attacker strength / Defender strength, expressed as ratio (e.g., 3:1)   | Most classic approach. Ratios are rounded in defender's favor.                  | Panzerblitz, Third Reich, most SPI games                                 |
| **Differential CRT**     | Attacker strength - Defender strength (e.g., +3)                        | Used for attrition-style combat where absolute numbers matter more than ratios. | NATO: The Next War in Europe, some tactical games                        |
| **Fire Table (non-CRT)** | Firepower value indexes a column; die roll determines result            | Used in tactical games. No ratio calculation. Each weapon fires independently.  | Advanced Squad Leader (Infantry Fire Table, To Hit Table, To Kill Table) |
| **Card-Driven Combat**   | Cards played from hand determine combat outcomes instead of dice on CRT | Eliminates dice entirely; hand management drives combat.                        | We The People, Hannibal: Rome vs. Carthage                               |
| **Hybrid**               | Dice + cards, or multiple tables in sequence                            | Combines elements; e.g., CDG event cards modify a hex-based CRT combat.         | Empire of the Sun, Paths of Glory                                        |

**CRT Structure:**

A CRT is a 2D matrix:

- **Columns**: Odds ratios (1:2, 1:1, 2:1, 3:1, 4:1, 5:1, 6:1+) or differential values (-3, -2, -1,
  0, +1, +2, +3...)
- **Rows**: Die roll results (typically 1-6 for 1d6, or 2-12 for 2d6)
- **Cells**: Result codes (see below)

**Common CRT Result Types:**

| Code | Full Name                  | Meaning                                                           |
| ---- | -------------------------- | ----------------------------------------------------------------- |
| AE   | Attacker Eliminated        | All attacking units destroyed                                     |
| AR   | Attacker Retreat           | Attacker must retreat X hexes                                     |
| AS   | Attacker Step Loss         | Attacker loses N steps                                            |
| EX   | Exchange                   | Both sides lose equal to defender's strength; defender eliminated |
| HEX  | Half Exchange              | Attacker loses half defender's strength                           |
| NE   | No Effect                  | Nothing happens                                                   |
| DR   | Defender Retreat           | Defender must retreat X hexes                                     |
| DS   | Defender Step Loss         | Defender loses N steps                                            |
| DE   | Defender Eliminated        | All defending units destroyed                                     |
| DRL  | Defender Retreat with Loss | Defender retreats AND takes losses                                |
| DD   | Defender Disrupted         | Defender flipped to disrupted side                                |
| AD   | Attacker Disrupted         | Attacker flipped to disrupted side                                |

**Combat Modifiers (applied before dice roll or as column shifts):**

| Modifier Type | Examples                                                                                             |
| ------------- | ---------------------------------------------------------------------------------------------------- |
| Terrain       | Defender in woods: 1 column shift left. Defender behind river: halve attacker. City: triple defense. |
| Combined Arms | Armor + infantry attacking together: 1 column shift right                                            |
| Flanking      | Attack from 3+ hexsides: column shift right per additional hexside                                   |
| Supply status | Out of supply: halve attack or defense                                                               |
| Unit quality  | Elite: +1 DRM. Green: -1 DRM                                                                         |
| Weather       | Mud: -2 column shift for attacker                                                                    |
| Fortification | Entrenched: double defense. Fort: triple defense                                                     |
| Leader/HQ     | In command radius: +1 DRM                                                                            |

**Commonality:** Universal (every wargame has some form of combat resolution).

**Data model implications:**

- CRT is a first-class data structure: columns (odds or differential values), rows (die results),
  cells (result codes).
- Result codes must be defined per game (each game has its own alphabet of results).
- Modifier system needs: source (terrain, unit property, weather, etc.), type (column shift, DRM,
  strength multiplier, strength halving), and magnitude.
- Must support both ratio and differential calculation modes.
- Must support multi-table resolution (ASL has ~200 tables).
- Must support non-CRT combat systems (card-driven, formula-based).
- Need post-combat procedures: retreat paths, advance after combat, exploitation triggers.

**Sources:** [Combat Results Table - Wikipedia](https://en.wikipedia.org/wiki/Combat_results_table),
[The Anvil of Probability](https://www.skeletoncodemachine.com/p/combat-results-table),
[Board Game Designers Forum: CRT](https://www.bgdf.com/forum/archive/archive-game-creation/game-design/combat-results-tables)

---

### 1.4 Turn Structure and Phasing

**How it works:** The game proceeds in discrete turns. Each turn is divided into phases executed in
a fixed order called the Sequence of Play. The most common structure is "I-Go-You-Go" (IGOUGO): one
player completes all phases, then the other player does the same.

**Standard IGOUGO Turn Structure:**

```
Game Turn N:
  Player A Turn:
    1. Reinforcement Phase (place new units)
    2. Movement Phase (move all units)
    3. Combat Phase (resolve all attacks)
    4. Exploitation Phase (special movement for breakthrough units) [if applicable]
    5. Supply Phase (check supply status)
    6. Administrative Phase (flip markers, advance turn)
  Player B Turn:
    [same phases repeated]
  End of Turn:
    Weather determination for next turn
    Random events
    Victory check
```

**Phase Variants:**

| Variant                    | Description                                                        | Example Games                                              |
| -------------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------- |
| **IGOUGO (basic)**         | Player A does everything, then Player B                            | Classic SPI/AH games                                       |
| **IGOUGO (interleaved)**   | Player A moves, Player B may react, then Player A attacks          | OCS (reaction phase)                                       |
| **Alternating Activation** | Players alternate activating single units or formations            | Many modern designs, miniatures crossovers                 |
| **Chit-Pull Activation**   | Random draw determines which formation activates next              | Combat Commander, The Gamers TCS                           |
| **Impulse**                | Turn divided into multiple impulses; each impulse both players act | Paths of Glory, many CDGs                                  |
| **Simultaneous**           | Both players write orders secretly, then execute simultaneously    | Diplomacy, some double-blind games                         |
| **Variable sub-phases**    | Number of action cycles per turn is variable/random                | GCACW (indefinite action cycles with initiative die rolls) |

**Commonality:** Universal (every game has turn structure; IGOUGO is the dominant paradigm).

**Data model implications:**

- Turn structure is a tree: Turn -> Player Turn -> Phase -> Sub-Phase -> Step.
- Each phase has: name, order, owning_player (or both), and a set of allowed actions.
- Must support both fixed-order and random/conditional phase sequences.
- Activation systems need: activation pool (chits/cards), eligibility rules, and exhaustion
  tracking.
- Variable-length turns need: end-condition checks after each impulse/cycle.
- Phase interleaving needs: reaction windows where the non-phasing player can act.

**Sources:**
[The Turn in Different Gaming Systems](https://brushandboltgun.com/2018/01/07/i-go-you-go-we-go-the-turn-in-different-gaming-systems/),
[OCS Sequence of Play v4.3](https://dornshuld.com/rules/ocs43/2-0-sequence-of-play.html)

---

### 1.5 Zones of Control (ZOC)

**How it works:** Most combat units project a Zone of Control into the six hexes surrounding them.
This represents the unit's ability to observe, threaten, and influence adjacent territory. ZOC
affects enemy movement and sometimes combat and supply.

**ZOC Types:**

| Type                   | Movement Effect                                                                | Example                                       |
| ---------------------- | ------------------------------------------------------------------------------ | --------------------------------------------- |
| **Rigid**              | Must stop upon entering enemy ZOC; cannot move ZOC-to-ZOC                      | Classic SPI games                             |
| **Semi-Rigid**         | Can enter enemy ZOC but cannot move directly from one enemy ZOC hex to another | Many modern operational games                 |
| **Fluid/Elastic**      | Entering enemy ZOC costs extra MPs but doesn't force a stop                    | Some post-2000 designs                        |
| **Locking**            | Cannot leave enemy ZOC once entered (must attack or be destroyed)              | Some older designs                            |
| **Selective**          | Only certain unit types project ZOC, or only against certain unit types        | ASL (only infantry projects ZOC in buildings) |
| **Negated by terrain** | ZOC doesn't extend across rivers, into mountains, or through certain features  | Many games                                    |

**ZOC Bond System (Simonitch):** The area between two friendly units separated by one hex creates a
"bond" that enemy units cannot pass through. This bond also blocks enemy supply and retreat. Used in
Normandy '44, Ardennes '44, Holland '44, Ukraine '43, France '40, Stalingrad '42, Salerno '43.

**ZOC Effects Beyond Movement:**

| Effect             | Description                                                         | Commonality            |
| ------------------ | ------------------------------------------------------------------- | ---------------------- |
| Supply blocking    | Supply lines cannot trace through enemy ZOC                         | Very common            |
| Retreat blocking   | Units retreating into enemy ZOC are eliminated or take extra losses | Very common            |
| Combat requirement | Units in enemy ZOC must attack (mandatory combat)                   | Common (classic games) |
| No advance         | Cannot advance after combat through enemy ZOC                       | Common                 |
| Reduced stacking   | Stacking penalties in enemy ZOC                                     | Uncommon               |

**Commonality:** Universal (present in virtually every hex wargame, though exact effects vary
enormously).

**Data model implications:**

- ZOC is a unit property: has_zoc (bool), zoc_type (enum), zoc_exceptions (terrain list, unit type
  list).
- ZOC bond system requires checking pairs of friendly units and the hex spine between them.
- ZOC effects must be configurable per game: movement cost, stop/no-stop, supply blocking, retreat
  interaction.
- Mutual ZOC interactions (both sides have ZOC in same hex) need specific rules.
- ZOC negation by terrain requires per-hexside or per-hex checks.

**Sources:** [Zone of Control - Wikipedia](https://en.wikipedia.org/wiki/Zone_of_control),
[WargameHQ: ZOC Basics](https://wargamehq.com/mechanics-monday-zone-of-control-basics/),
[ZOC Bond System - BGG](https://boardgamegeek.com/boardgamefamily/69195/series-zoc-bond-system)

---

### 1.6 Stacking Rules

**How it works:** Stacking limits restrict how many units can occupy a single hex. Limits are
typically expressed in number of units, number of steps, or total stacking points.

**Stacking Limit Types:**

| Type                | Example                                                        | Games                                     |
| ------------------- | -------------------------------------------------------------- | ----------------------------------------- |
| **Unit count**      | Max 3 units per hex                                            | Simple operational games                  |
| **Step count**      | Max 6 steps per hex (a 3-step division + 3 one-step regiments) | Some SPI games                            |
| **Stacking points** | Division=3 pts, regiment=2, battalion=1; max 6 pts/hex         | OCS, many GMT games                       |
| **Unit size**       | Max 1 division or 3 regiments per hex                          | Common variant                            |
| **No stacking**     | Exactly 1 unit per hex                                         | Some tactical games, Columbia block games |
| **Unlimited**       | No limit (rare in hex games)                                   | Very rare                                 |

**Additional stacking rules:**

- Stacking is usually checked at end of movement (units may move through overstacked hexes).
- Markers, HQs, and non-combat units often don't count for stacking.
- Some games have different stacking limits for different terrain (fewer units in mountains).
- Overstacking penalties: excess units eliminated, or defense penalty.

**Commonality:** Universal.

**Data model implications:**

- Each game defines: stacking_metric (units, steps, points), stacking_limit (number),
  stacking_exceptions (unit types that don't count).
- Terrain may modify stacking limits.
- Stacking check timing must be configurable (end of movement, end of combat, always).
- Need to handle transient overstacking during movement.

**Sources:**
[BGG: Stacking in Hex/Counter Games](https://boardgamegeek.com/thread/755832/can-someone-help-explain-hexcounter-games-and-stac)

---

### 1.7 Terrain System

**How it works:** Each hex contains a primary terrain type, and hexsides may have their own terrain
features. Terrain affects movement cost, combat strength modifiers (usually benefiting the
defender), line of sight, supply tracing, stacking, and more.

**Common Hex Terrain Types:**

| Terrain        | Movement Effect               | Combat Effect                               |
| -------------- | ----------------------------- | ------------------------------------------- |
| Clear/Open     | 1 MP                          | No modifier (baseline)                      |
| Woods/Forest   | 2 MP (inf), 3 MP (mech)       | +1 column shift for defender                |
| City/Urban     | 1-2 MP                        | Double or triple defense                    |
| Mountain       | 3 MP (inf), prohibited (mech) | Double defense; blocks LOS                  |
| Swamp/Marsh    | 2 MP (inf), prohibited (mech) | +1 shift for defender; limits fortification |
| Desert         | 1 MP                          | No modifier; supply costs increased         |
| Rough/Broken   | 2 MP                          | +1 shift for defender                       |
| Hill/Elevation | 2 MP                          | +1 shift; LOS advantage                     |

**Common Hexside Terrain:**

| Terrain            | Effect                                                       |
| ------------------ | ------------------------------------------------------------ |
| River (minor)      | +1 MP to cross; attacker halved if attacking across          |
| River (major)      | Prohibits mech crossing without bridge; attacker at 1/3      |
| Road               | Reduces hex cost to road rate when entering via road hexside |
| Railroad           | Enables rail movement when hex has rail capacity             |
| Ridge/Escarpment   | Blocks LOS; +1 MP; defensive bonus                           |
| Bridge             | Allows road-rate crossing of river; can be destroyed         |
| Fortification line | Permanent defensive bonus                                    |

**Terrain Effects Chart (TEC) Structure:** A TEC is typically a matrix with:

- Rows: Terrain types (clear, woods, city, mountain, etc.)
- Columns grouped into: Movement Cost (by unit type), Combat Effect (attacker modifier, defender
  modifier), Other (LOS, supply, stacking)

**Commonality:** Universal.

**Data model implications:**

- Each hex has: primary_terrain_type, elevation, and a set of hex features (road, rail, bridge,
  airfield, port, supply source, etc.).
- Each hexside has: hexside_terrain_type (river, road, ridge, wall, etc.).
- TEC is a core data table: terrain_type x effect_category x unit_type -> value.
- Terrain can have multiple overlapping effects (a hill hex with woods and a road).
- Elevation must be tracked as an integer for LOS calculations.
- Terrain types themselves must be definable per game (not a fixed set).

**Sources:**
[Terrain Effects Chart - Pushing Cardboard](https://pushingcardboard.com/base/glossary/14-terrain-effects-chart)

---

### 1.8 Unit Attributes and Counter Design

**How it works:** Each unit counter encodes its capabilities as numeric values and symbols. The
standard layout uses NATO military symbology for unit type and size, with numeric combat and
movement values.

**Standard Counter Layout:**

```
+-------------------+
|  [Unit ID/Name]   |
|  [NATO Symbol]    |
|  [Size Symbol]    |
|  [Parent Unit]    |
+-------------------+
| ATK | DEF |  MOV  |
+-------------------+
```

**Core Attributes:**

| Attribute          | Description                                                            | Universal?  |
| ------------------ | ---------------------------------------------------------------------- | ----------- |
| Attack Strength    | Numeric value for offensive combat                                     | Yes         |
| Defense Strength   | Numeric value for defensive combat                                     | Yes         |
| Movement Allowance | Movement points available per turn                                     | Yes         |
| Unit Type          | Infantry, Armor, Artillery, etc. (NATO symbol)                         | Yes         |
| Unit Size          | Squad, Platoon, Company, Battalion, Regiment, Brigade, Division, Corps | Yes         |
| Unit ID            | Historical designation (e.g., "2/506 PIR")                             | Yes         |
| Parent Formation   | Higher echelon (e.g., "101st Airborne")                                | Very common |
| Nationality/Side   | Which player controls the unit                                         | Yes         |

**Extended Attributes (common but not universal):**

| Attribute                  | Description                                            | Commonality                   |
| -------------------------- | ------------------------------------------------------ | ----------------------------- |
| Action Rating / Quality    | Numeric rating reflecting training, experience, morale | Common (OCS, ASL)             |
| Steps                      | Number of damage increments before elimination         | Very common                   |
| Barrage Strength           | Strength for ranged bombardment attacks                | Common (operational)          |
| Range                      | Firing/barrage range in hexes                          | Common (tactical/operational) |
| Anti-Tank / Anti-Personnel | Separate values for different target types             | Common (tactical)             |
| Stacking Points            | Value for stacking calculations                        | Common                        |
| Supply Cost                | Supply draw per turn                                   | Uncommon                      |
| Construction Value         | Ability to build fortifications                        | Uncommon                      |

**NATO Unit Type Symbols:**

| Symbol            | Type                | Symbol                  | Type      |
| ----------------- | ------------------- | ----------------------- | --------- |
| X                 | Infantry            | Rectangle with diagonal | Armor     |
| Dot in rectangle  | Mechanized Infantry | Circle                  | Artillery |
| Slashed rectangle | Cavalry             | ~                       | Naval     |
| Winged            | Airborne            | Cross                   | Engineer  |
| Binoculars        | Recon               | Star                    | HQ        |

**NATO Size Symbols:**

| Symbol | Size            | Symbol | Size       |
| ------ | --------------- | ------ | ---------- |
| .      | Fire Team       | ..     | Squad      |
| ...    | Section         | I      | Platoon    |
| II     | Company/Battery | III    | Battalion  |
| X      | Regiment/Group  | XX     | Brigade    |
| XXX    | Division        | XXXX   | Corps      |
| XXXXX  | Army            | XXXXXX | Army Group |

**Step Reduction Methods:**

| Method                                 | Description                                                                  | Games                              |
| -------------------------------------- | ---------------------------------------------------------------------------- | ---------------------------------- |
| Flip (2-step)                          | Full strength front / reduced back. Flip on first loss, eliminate on second. | Most hex wargames                  |
| Multi-counter replacement              | Unit has 3-4 counters at different strengths; swap counters on loss.         | Some SPI games, Illusions of Glory |
| Strength track                         | Separate track records current strength points (10, 9, 8...)                 | Some digital implementations       |
| Block rotation                         | Wooden block rotated to show reduced pip count at top.                       | Columbia Games                     |
| Multi-track (cohesion/strength/morale) | Separate tracks for formation hits, morale hits, and casualty points.        | GBoH, some tactical games          |

**Commonality:** Universal (every game has units with numeric attributes).

**Data model implications:**

- Unit is the central entity with extensible attributes (different games need different attributes).
- Must support a "unit template" or "counter manifest" that defines all unit types for a game.
- Each unit needs: current values (possibly reduced) and original/full-strength values.
- Step tracking needs: max_steps, current_steps, and per-step attribute values (reduced side may
  have different ATK/DEF/MOV).
- Organizational hierarchy: unit -> parent formation -> higher formation (for command/integrity
  bonuses).
- Must support custom attributes per game (any game might invent new unit properties).

**Sources:** [WargameHQ: Counter Layout](https://wargamehq.com/mechanic-monday-counter-layout/),
[BGG: NATO Counter Symbols Guide](https://boardgamegeek.com/thread/222132/guide-military-map-and-wargaming-counter-symbols)

---

### 1.9 Victory Conditions

**How it works:** The game defines how a winner is determined. Victory conditions vary widely but
typically involve controlling key locations, destroying enemy units, or achieving specific
objectives by a deadline.

**Victory Condition Types:**

| Type                     | Description                                                                                 | Commonality                |
| ------------------------ | ------------------------------------------------------------------------------------------- | -------------------------- |
| **Hex control**          | Control specific key hexes at game end (e.g., cities, crossroads)                           | Very common                |
| **Victory Points (VP)**  | Accumulate VP from controlling hexes, destroying units, events; highest wins                | Very common                |
| **Sudden death**         | Immediate win if a specific condition is met (e.g., capital captured)                       | Common                     |
| **Graduated victory**    | Multiple levels: decisive victory, marginal victory, draw, marginal defeat, decisive defeat | Very common                |
| **Territorial**          | Control X% of map area or specific regions                                                  | Common                     |
| **Attrition**            | Destroy X enemy units or reduce enemy below threshold                                       | Uncommon as sole condition |
| **Political track**      | Reach certain level on an abstract political/morale track                                   | Common (CDGs, COIN)        |
| **Variable game length** | Game might end randomly after turn X; players must optimize for uncertain endpoint          | Common                     |
| **Asymmetric**           | Each side has different victory conditions                                                  | Common                     |
| **Multi-faction**        | Each of 3-4 factions has unique victory conditions checked independently                    | COIN series                |

**Commonality:** Universal.

**Data model implications:**

- Victory conditions are per-scenario, not per-game-system.
- Need to support: hex control checks, VP tallies, threshold comparisons, per-side conditions, and
  timing (checked every turn, at game end, or continuously for sudden death).
- Graduated victory requires ordered victory levels with threshold ranges.
- Asymmetric conditions require per-player/per-faction condition sets.

---

## Area 2: Advanced/Common Mechanics

These appear in many but not all wargames and add significant depth.

---

### 2.1 Supply and Logistics

**How it works:** Units must trace a supply line from their location to a supply source. Being out
of supply degrades combat effectiveness and may cause attrition. Some games further require specific
supply expenditure (fuel, ammunition) for specific actions.

**Supply System Variants:**

| Variant                    | Description                                                                                                          | Example Games                               |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| **Binary trace supply**    | Unit can or cannot trace a path of N hexes to a supply source. In or out.                                            | Most SPI operational games                  |
| **Graduated supply**       | Multiple supply states (full, partial, out of supply, isolated) with increasing penalties                            | Some GMT games                              |
| **Physical supply tokens** | Supply represented as on-map counters that must be physically moved (by truck, rail, etc.) from sources to consumers | OCS (the definitive physical supply system) |
| **Supply radius from HQ**  | HQs have a supply radius; units within radius are supplied; HQs trace to source                                      | Many operational games                      |
| **Fuel + Ammo separation** | Different supply types: fuel needed to move mechanized units; ammo/combat supply needed to fight                     | OCS                                         |
| **Supply line tracing**    | Path must follow roads/rails back to a port or map-edge source; length limits apply                                  | Very common                                 |

**OCS Supply Detail:**

- **Trace supply**: Abstract; units trace path to HQ which traces to rail/port. Being "out of
  supply" halves combat, removes ZOC, and prevents attack.
- **Combat supply**: On-map supply point tokens (SPs) physically expended to attack. 1 SP per 2
  attacking units.
- **Fuel**: On-map SPs expended to move tracked/truck units. Required for mechanized movement.
- **Barrage supply**: SPs expended for artillery bombardment.
- **Out of Supply effects**: No ZOC; attack at half (with combat supply) or cannot attack (without);
  defend at half or quarter.

**Commonality:** Common to very common. Some form of supply appears in most operational+ games. Full
physical supply (OCS-style) is uncommon.

**Data model implications:**

- Supply sources: hexes tagged as supply_source (ports, depots, map edges).
- Supply tracing: pathfinding algorithm that checks for valid hex paths of limited length, not
  blocked by enemy units/ZOC.
- HQ-based supply: HQ units have supply_radius and supply_distribution_value.
- Physical supply: supply tokens as on-map entities with movement, stacking, and consumption rules.
- Supply status per unit: supply_state enum (in_supply, partial_supply, out_of_supply, isolated).
- Effects of supply states must modify unit capabilities (attack, defense, movement, ZOC).

**Sources:**
[How Wargames Model Logistics](https://www.meeplemountain.com/articles/how-wargames-model-logistics/),
[OCS Supply Rules v4.3](https://dornshuld.chemistry.msstate.edu/rules/ocs43/12-0-supply.html),
[Deep Battle Design Notes on Logistics](https://balagan.info/deep-battle-design-notes-4-musing-on-logistics-and-supply-rules)

---

### 2.2 Command and Control

**How it works:** Units must be "in command" to operate at full effectiveness. Command is typically
established by proximity to a headquarters (HQ) unit, which itself must be connected to higher HQs
in a chain of command.

**Command Mechanics:**

| Mechanic                       | Description                                                                         | Commonality          |
| ------------------------------ | ----------------------------------------------------------------------------------- | -------------------- |
| **Command radius**             | HQ has a radius in hexes; units within radius are "in command"                      | Very common          |
| **Chain of command**           | Division HQ -> Corps HQ -> Army HQ; each must be in range of the next               | Common               |
| **Out of command penalties**   | Halved movement, halved attack, cannot receive replacements                         | Common               |
| **Activation limits**          | HQ can only activate N units per turn                                               | Uncommon             |
| **Command points**             | HQ generates command points; activating units costs points                          | Uncommon             |
| **Leader quality**             | Individual leader counter with quality ratings affecting nearby units               | Common (GCACW, GBoH) |
| **Regimental Integrity Bonus** | Units from the same formation attacking together get combat bonuses (5 DRM in GOSS) | Common               |

**Commonality:** Common (operational and strategic games). Absent from many simple tactical games.

**Data model implications:**

- HQ units need: command_radius, command_capacity, command_quality.
- Units need: parent_formation reference for integrity bonuses.
- Command chain: tree structure of HQ -> subordinate HQs -> combat units.
- "In command" is a computed status based on distance from HQ.
- Command status modifies unit capabilities similarly to supply status.

---

### 2.3 Morale and Cohesion

**How it works:** Units have a morale or cohesion value that degrades under combat stress. When
morale fails, units break, rout, or suffer penalties. Morale is separate from physical losses.

**Morale State Progressions (typical):**

```
Normal -> Disrupted -> Demoralized -> Routed -> Eliminated/Surrendered
```

Or in OCS terms:

```
Normal -> Disorganized (DG) -> [eliminated by further combat]
```

**Morale Mechanics:**

| Mechanic                    | Description                                                                                | Example Games                  |
| --------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------ |
| **Morale check**            | Roll against morale value; failure causes state change (broken, routed)                    | ASL, GBoH, most tactical games |
| **Disruption/DG**           | Binary flag; disrupted units halve all values                                              | OCS                            |
| **Cascading rout**          | When one unit routs, adjacent friendly units must check morale; can cause chain reaction   | GBoH, many tactical games      |
| **Rally**                   | Disrupted/routed units can attempt to recover during rally phases                          | Nearly all games with morale   |
| **Unit quality/experience** | Veteran units are harder to break; green troops break easily                               | ASL, OCS (Action Rating)       |
| **Formation cohesion**      | A formation-level track degrades as units take losses; when depleted, the formation breaks | GBoH                           |
| **Drive value**             | Each unit has a "Drive" that drops with casualties; at zero the unit routs                 | Some tactical games            |

**Commonality:** Common. Some form appears in most games; ranges from simple (OCS DG marker) to
complex (ASL multi-level morale with ELR, self-rally, berserk, etc.).

**Data model implications:**

- Units need: morale_value (or action_rating), current_morale_state (enum: normal, disrupted,
  demoralized, routed, broken).
- Morale state machine: define valid transitions and triggers per game.
- Morale checks: threshold comparison (die roll vs. morale value) with modifiers.
- Rally: phase-based recovery mechanic; requires proximity to leaders/HQs.
- Formation-level cohesion: aggregate tracker across units in a formation.

**Sources:**
[Hollandspiele: Failing My Morale Check](https://hollandspiele.com/blogs/hollandazed-thoughts-ideas-and-miscellany/failing-my-morale-check),
[BGG: Wargames with Morale](https://boardgamegeek.com/geeklist/4697/principles-strategy-6-wargames-morale)

---

### 2.4 Weather Effects

**How it works:** Weather changes across turns, affecting movement costs, combat modifiers, air
operations, and supply. Weather is typically determined randomly at the start of each turn via a
weather table.

**Weather Dimensions:**

| Dimension              | Values                            | Effects                                                    |
| ---------------------- | --------------------------------- | ---------------------------------------------------------- |
| **Ground conditions**  | Dry, Mud, Snow, Frozen, Deep Snow | Movement cost multipliers; mech movement restricted in mud |
| **Air/Sky conditions** | Clear, Overcast, Storm, Fog       | Air operations limited/prohibited; visibility reduced      |
| **Temperature**        | Normal, Cold, Extreme Cold        | Attrition; river crossing changes (frozen rivers)          |

**Complex Weather (WitE2):** Tracks weather per zone (8 zones), per hex, with weather fronts that
move across the map. Six ground conditions and six air conditions per zone.

**Commonality:** Common in operational/strategic games. Rare in tactical games.

**Data model implications:**

- Weather state: per-turn (simple) or per-zone/per-hex (complex).
- Weather table: turn number or season -> die roll -> weather result.
- Weather effects: modify TEC entries, supply range, air availability, attrition.
- Seasonal progression: weather probabilities change by month/season.

---

### 2.5 Fog of War / Limited Intelligence

**How it works:** Players have incomplete information about enemy forces, simulated through various
mechanics.

**Fog of War Methods:**

| Method                 | Description                                                                       | Commonality                             |
| ---------------------- | --------------------------------------------------------------------------------- | --------------------------------------- |
| **Face-down counters** | Counters placed upside down; identity revealed on contact                         | Common                                  |
| **Dummy counters**     | Empty counters mixed with real units; opponent can't tell which is which          | Common                                  |
| **Hidden deployment**  | Players record initial positions secretly; reveal on contact                      | Common                                  |
| **Block games**        | Wooden blocks stand upright facing the owner; opponent sees only the back         | Columbia Games series                   |
| **Double-blind**       | Each player has own map; referee mediates contact/sighting                        | Uncommon (requires referee or computer) |
| **Numbered counters**  | All counters look identical; numbers track which is which on a private roster     | Uncommon                                |
| **Concealment**        | Units in certain terrain get concealment markers; must be spotted before attacked | ASL                                     |

**Commonality:** Common (some form exists in many games, but full hidden information is harder in
face-to-face board games).

**Data model implications:**

- Units need: visibility_state (hidden, concealed, revealed, dummy).
- Per-player visibility: each player sees different information about the same hex.
- Digital implementation advantage: hidden info is trivial compared to physical games.
- Must support reveal triggers (adjacent enemy, combat, reconnaissance).
- Dummy units: entities that appear to be real units but have no combat value.

**Sources:**
[Fog of War in Tabletop Games](https://therewillbe.games/blogs-by-members/2983-blank-36937551),
[Pulsiphergames: Ways to Reflect Fog of War](https://pulsiphergames.com/gamedesign/WaysToReflectFogofWar.htm)

---

### 2.6 Air Power

**How it works:** Air units represent squadrons, wings, or groups. They typically operate
differently from ground units -- placed in missions rather than moving hex by hex. Air power
provides ground support, interdiction, air superiority, and strategic bombing.

**Air Mission Types:**

| Mission                         | Effect                                                     | Commonality              |
| ------------------------------- | ---------------------------------------------------------- | ------------------------ |
| **Close Air Support (CAS)**     | Adds strength to ground combat as column shift or DRM      | Very common              |
| **Interdiction**                | Increases movement cost in target area; blocks supply      | Common                   |
| **Air Superiority**             | Contests enemy air missions; enables/denies other missions | Common                   |
| **Strategic Bombing**           | Attacks production, rail capacity, ports                   | Common (strategic games) |
| **Reconnaissance**              | Reveals hidden enemy units                                 | Common                   |
| **Transport/Airlift**           | Moves supply or units by air                               | Common                   |
| **Air ZOI (Zone of Influence)** | Blocks supply/communication through airspace               | Empire of the Sun        |

**Air Unit Handling:**

- Often placed on an "air display" rather than on the map.
- Assigned to missions during an air phase.
- "Flipped" or "fatigued" after use; need a rest turn to become available again.
- Allocation is often a limited resource (X air points per turn).

**Commonality:** Common in operational and strategic games.

**Data model implications:**

- Air units may live off-map (air display, air base, carrier).
- Missions as a concept: assignment of air units to targets/areas with specific mission types.
- Air availability: pool-based resource (N air points per turn) rather than per-unit movement.
- Anti-air: ground units or facilities that reduce air effectiveness.

---

### 2.7 Artillery and Indirect Fire

**How it works:** Artillery units can attack targets at range without being adjacent. This is
resolved through bombardment/barrage tables rather than the standard CRT.

**Artillery Mechanics:**

| Mechanic                | Description                                                                         | Commonality       |
| ----------------------- | ----------------------------------------------------------------------------------- | ----------------- |
| **Barrage strength**    | Separate value from attack strength; used on barrage table                          | Common (OCS)      |
| **Range**               | Can fire N hexes away                                                               | Common            |
| **Barrage table**       | Separate CRT for bombardment; results usually disruption/DG rather than elimination | Common            |
| **Counterbattery fire** | Enemy artillery can target your artillery during bombardment                        | Common            |
| **Forward observer**    | Needs friendly unit adjacent to target to "spot" for indirect fire                  | Common (tactical) |
| **Fire support**        | Artillery "supports" an attack by adding its barrage value to the combat            | Very common       |
| **Ammunition**          | Limited ammo supply; each fire mission costs supply                                 | Common (OCS)      |

**Commonality:** Common. Present in most operational games and all tactical games.

**Data model implications:**

- Artillery units need: barrage_strength, range, ammo/supply_cost.
- Barrage table: separate from main CRT; indexed by barrage strength and die roll.
- Targeting: must check range, LOS (if required), and spotter requirements.
- Integration with ground combat: artillery can support attacks as a modifier.

---

### 2.8 Fortifications and Entrenchment

**How it works:** Units can build defensive positions over time, gaining increasing defensive
bonuses. Permanent fortifications (like the Maginot Line or Siegfried Line) are pre-printed on the
map.

**Fortification Levels:**

| Level | Name                                    | Typical Bonus                               | Build Time     |
| ----- | --------------------------------------- | ------------------------------------------- | -------------- |
| 0     | None                                    | --                                          | --             |
| 1     | Hasty defense / foxholes                | +1 DRM or minor shift                       | 1 turn         |
| 2     | Improved position / trenchworks         | +2 DRM or defense doubled                   | 2-3 turns      |
| 3     | Prepared defense                        | +3 DRM or defense tripled                   | Many turns     |
| 4-5   | Permanent fortification (Maginot, etc.) | Massive bonus; may be impervious to assault | Pre-built only |

**Construction factors:** Unit must be stationary; engineer units build faster; terrain and supply
affect construction rate. In WitE2, each unit has a construction_value based on unit type,
experience, and fatigue, and construction progress accumulates each turn.

**Commonality:** Common. Simple entrenchment is very common; multi-level fortification is less so.

**Data model implications:**

- Per-hex: fortification_level (integer).
- Construction rules: units have construction_value; accumulates toward next level's threshold.
- Fortification effects: modify TEC combat modifiers for the hex.
- Engineer units: construction_multiplier bonus.

---

### 2.9 Replacements, Reinforcements, and Withdrawals

**How it works:** New units enter the game according to a schedule; existing units can be rebuilt by
spending replacement points; and units may be removed from play on schedule.

**Terminology:**

| Term              | Meaning                                                                |
| ----------------- | ---------------------------------------------------------------------- |
| **Reinforcement** | A new unit entering the game for the first time                        |
| **Replacement**   | Strength points used to rebuild existing reduced units                 |
| **Withdrawal**    | A unit required to leave the map on a specific turn                    |
| **Rebuild**       | A destroyed unit that returns to play (may require replacement points) |

**Reinforcement Schedule Format:**

```
Turn 3: 2/506 PIR (101st Abn) enters at hex 2413 or 2414
Turn 5: 10th Armored Div enters at any west map edge road hex
Turn 7: 501 PIR withdrawn from play
```

**Replacement System Variants:**

- **Replacement points**: Receive N points per turn; spend 1 point per step to rebuild a reduced
  unit.
- **Step recovery**: Reduced units automatically recover 1 step per turn if in supply and not in
  enemy ZOC.
- **Reorganization**: Eliminated units can be reformed from remnants after a delay.

**Commonality:** Very common. Reinforcement schedules appear in virtually every scenario-based game.

**Data model implications:**

- Reinforcement schedule: ordered list of (turn, unit, entry_hex_or_zone, conditions).
- Withdrawal schedule: ordered list of (turn, unit).
- Replacement pool: resource tracked per side per turn.
- Unit states: on_map, in_reserve, eliminated, withdrawn, awaiting_reinforcement.
- Entry hexes/zones: tagged hexes for reinforcement placement.

---

### 2.10 Scale Differences: Tactical, Operational, Strategic

**How it works:** Wargames operate at different scales, which fundamentally affects which mechanics
are relevant.

| Scale               | Hex Size   | Turn Length      | Unit Size                     | Key Mechanics                                               |
| ------------------- | ---------- | ---------------- | ----------------------------- | ----------------------------------------------------------- |
| **Tactical**        | 50-200m    | Minutes to hours | Squad, platoon, company       | LOS, fire tables, morale checks, individual weapons, facing |
| **Operational**     | 2-10 mi    | Days to weeks    | Battalion, regiment, division | Supply, exploitation, breakthrough, command radius          |
| **Strategic**       | 20-100+ mi | Weeks to months  | Division, corps, army         | Production, diplomacy, strategic bombing, national morale   |
| **Grand Strategic** | 100+ mi    | Months to years  | Army, army group, theater     | Political tracks, alliance management, technology research  |

**Commonality:** Universal distinction (every game sits at a specific scale).

**Data model implications:**

- The tool must be scale-agnostic: all mechanics exist on a spectrum, not in fixed scale buckets.
- Some mechanics only apply at certain scales but the tool shouldn't enforce this -- let designers
  compose freely.
- Scale determines which attributes are relevant: tactical needs weapon ranges and LOS; operational
  needs supply chains; strategic needs production and diplomacy.

**Sources:**
[Four Levels of Wargaming](https://www.beastsofwar.com/featured/levels-wargaming-part-3-operational-level/),
[BGG: Wargame Scales](https://boardgamegeek.com/thread/1820050/wargame-scaleswhat-are-they)

---

## Area 3: Bespoke/Unusual Mechanics

These are found in specific games or game families and represent the edges of the design space.

---

### 3.1 Chit-Pull Activation Systems

**How it works:** Instead of alternating full player turns, formation chits are placed in a cup. A
chit is drawn randomly; that formation activates (moves and/or fights). This continues until all
chits are drawn (end of turn). Provides uncertainty about when each formation will act.

**Variants:**

- **Formation activation**: Each chit represents a division/corps. All units in that formation
  activate.
- **Action type activation**: Chits represent "Move," "Fire," "Rally" -- the drawn action applies to
  any units.
- **Quality-based**: Better formations have more chits in the cup (more likely to activate, possibly
  multiple times).
- **Event chits**: Some chits trigger random events instead of activating units.
- **End-of-turn chit**: A special chit that ends the turn immediately when drawn, leaving remaining
  formations unactivated.

**Example games:** Combat Commander, Ardennes '44 (optional), The Gamers' Tactical Combat Series
(TCS), many GMT games.

**Commonality:** Common (growing trend; increasingly popular since 1990s).

**Data model implications:**

- Activation pool: collection of chit definitions with draw probabilities.
- Each chit has: activation_target (formation, player, action type, event), and optional conditions.
- Must support multiple draws per turn and variable turn length (end-of-turn chit).
- Solo-friendly: chit-pull inherently provides decision-making for the non-active side.

**Sources:**
[BGG: Chit-Pull System](https://boardgamegeek.com/boardgamemechanic/2057/chit-pull-system),
[The Players' Aid: Best Chit-Pull Games](https://theplayersaid.com/2020/02/21/best-3-games-with-chit-pull-activation/)

---

### 3.2 Card-Driven Game (CDG) Mechanics

**How it works:** Players hold a hand of cards. Each card can be used either for its printed event
OR for its operations value (to move/fight). This creates a constant tension between using powerful
events and spending ops points. First introduced in We The People (1994) by Mark Herman.

**CDG Core Concepts:**

- **Dual-use cards**: Play as event OR as operations points.
- **Operations points**: Used to move X units, make coups, place influence, etc.
- **Events**: Named historical events with specific effects (mandatory or optional).
- **Opponent events**: Some cards have events that benefit the opponent; you must still play them,
  triggering the opponent's event.
- **Card cycling**: Deck is reshuffled when exhausted; some events are removed after play.

**Evolution:**

- We The People (1994): First CDG; card-driven combat (Battle Cards replace CRT).
- Hannibal: Rome vs. Carthage (1996): Dual-use Strategy Cards; separate Battle Cards.
- Paths of Glory (1999): CDG on a hex map with traditional combat.
- Twilight Struggle (2005): CDG for political influence, not military operations.
- Empire of the Sun (2005): First CDG to fully integrate with hex-based operational wargame.

**Commonality:** Common (major design tradition since 1994; dozens of CDGs published).

**Data model implications:**

- Card definition: name, ops_value, event_text, event_effects, owning_side (or neutral),
  removal_after_play (bool).
- Hand management: draw_count, hand_limit, discard rules.
- Card play: choice between event and operations.
- Event effects: need a scripting/effect system to encode arbitrary card effects.
- Some CDGs are point-to-point rather than hex, but the card system applies to both.

**Sources:**
[A Brief History of Card-Driven Wargames](https://www.meeplemountain.com/articles/a-brief-history-of-card-driven-wargames/),
[Mark Herman: What is a CDG?](https://markherman.tripod.com/blog/index.blog/2034771/what-is-a-cdg/)

---

### 3.3 Overrun Mechanics

**How it works:** During movement, a strong unit can "overrun" a weak enemy unit in its path,
attacking on the move without stopping. This is resolved immediately and the unit continues moving
if successful.

**Overrun Rules (OCS example):**

- One overrun attempt per target hex per movement phase.
- Each moving stack can only conduct one overrun per phase.
- Costs movement points (typically 3 MPs) plus combat supply.
- Uses CRT at favorable odds; if successful, defender retreats or is eliminated and attacker
  continues.
- Failure may result in attacker becoming disorganized.

**Commonality:** Common (most operational games have some form of overrun).

**Data model implications:**

- Overrun is a movement-phase combat action (not in the combat phase).
- Needs: cost_in_MPs, supply_cost, minimum_odds, CRT_reference, success/failure_effects.
- Must interrupt normal movement resolution to resolve combat mid-move.

---

### 3.4 Exploitation and Breakthrough Combat

**How it works:** After successful combat, certain units (especially armor) can make additional
moves and attacks in a special exploitation phase, simulating the "blitzkrieg" effect of breaking
through and exploiting gaps.

**Exploitation Variants:**

- **Exploitation Phase**: A separate phase after the combat phase where qualifying units (those that
  earned exploitation mode) can move and fight again.
- **Breakthrough Combat**: Units that advance after combat may immediately attack again into the
  next hex.
- **Exploitation Mode**: Triggered by specific CRT results (e.g., a DE result at high odds).
- **Mechanized Movement Phase**: Only mechanized units can move in this additional phase.

**Example games:** OCS (Exploitation Phase with Exploitation Mode markers), Stalingrad '42
(Breakthrough Combat), many WWII operational games.

**Commonality:** Common in WWII operational games. Rare in other eras.

**Data model implications:**

- Unit state: exploitation_eligible (bool), set by CRT result.
- Additional phase in sequence of play: exploitation_phase with restricted unit eligibility.
- Breakthrough combat: secondary attack triggered by advance after combat.
- Mechanized-only movement: filtered movement phase.

---

### 3.5 Reserve Commitment

**How it works:** Units placed in "Reserve Mode" are held back from regular operations but can be
released in response to enemy actions (during the enemy's turn). This simulates the critical
decision of when to commit reserves.

**OCS Reserve Mode:**

- Reserve units cannot attack, overrun, or barrage until released.
- Defend at half combat strength if attacked while in reserve.
- Can only move 1/4 MA during regular movement.
- During enemy Reaction Phase, reserve units can be released to move at 1/2 MA and fight.
- Once released, they lose Reserve Mode and adopt their underlying mode.

**Commonality:** Common (particularly in operational games).

**Data model implications:**

- Unit mode/state: reserve_mode with specific capability restrictions.
- Reaction phase: a window during the opponent's turn where reserve units can be committed.
- Release conditions: player choice during enemy phase, or triggered by enemy actions in range.

---

### 3.6 Reaction/Opportunity Fire

**How it works:** During the enemy's movement or combat phase, defending units can interrupt to fire
at moving enemy units. This is critical in tactical games where suppression and ambush matter.

**Variants:**

- **Defensive Fire**: Defender fires back during enemy combat resolution.
- **Opportunity Fire**: Units fire at enemy units moving through their range/LOS. Each firing unit
  may be limited to N shots.
- **Overwatch**: Units set to "overwatch" mode during their turn; they fire automatically at enemies
  moving in their field of fire during the enemy turn.
- **Final Protective Fire**: All units in a hex fire simultaneously as a last-ditch defense when
  assaulted.

**Example games:** ASL (Defensive Fire, Final Protective Fire), Squad Leader, Panzer, most tactical
games.

**Commonality:** Very common in tactical games. Uncommon at operational scale (OCS reaction movement
is the equivalent).

**Data model implications:**

- Interruption system: enemy actions can trigger friendly reaction fire.
- Reaction eligibility: range, LOS, remaining shots, overwatch status.
- Shot tracking: units have N fire opportunities per phase.
- Fire resolution: uses fire tables (IFT, TH, TK) rather than standard CRT.

---

### 3.7 Hidden Units and Dummy Counters

**How it works:** Real units are mixed with dummy (empty) counters. The opponent cannot tell which
is which until contact is made or reconnaissance reveals them.

**Implementation Methods:**

- 2 dummy counters per real unit placed during setup.
- All counters face-down; flipped when entering enemy LOS or ZOC.
- Numbered counters: all look the same; real identity tracked on private roster.
- "Blinds" or hidden movement markers: markers move on map; resolved when "spotted."

**Example games:** Rommel in the Desert, many ASL scenarios (concealment counters), block games.

**Commonality:** Common (in various forms). Full hidden movement is less common in hex-and-counter.

**Data model implications:**

- Dummy counter type: entity with no real combat value but occupies hex and moves.
- Reveal trigger: enemy unit adjacent, LOS check, recon mission, or combat.
- Per-player visibility model: each player sees different counter information.
- Concealment markers: overlay that grants defensive bonus and hides identity.

---

### 3.8 Variable Turn Length / Sudden Death

**How it works:** Instead of a fixed number of turns, the game might end randomly after a certain
point. Players cannot count on having all scheduled turns.

**Variants:**

- **Random end check**: After a designated turn, roll a die each turn; on certain results, the game
  ends immediately.
- **Sudden death condition**: Game ends instantly when a specific condition is met (capital falls,
  army destroyed).
- **Variable action cycles**: Within a turn, the number of action rounds is random (GCACW).

**Example games:** Many modern GMT games, Warhammer 40K (turns 5-7 random), GCACW.

**Commonality:** Common.

**Data model implications:**

- Turn end condition: per-turn check (die roll, condition, or fixed).
- Must support both fixed-length and variable-length games.
- Victory condition evaluation must work at any turn (not just a predetermined "final turn").

---

### 3.9 Political/Diplomatic Tracks

**How it works:** Abstract tracks represent non-military dimensions of the conflict: political
support, diplomatic relations, war weariness, national morale, international opinion.

**Examples:**

- **Twilight Struggle**: DEFCON track (nuclear risk), Space Race track, regional scoring.
- **COIN series**: Population Support/Opposition tracks per region; faction resource and victory
  tracks.
- **Paths of Glory**: War Status track per power (mobilization through surrender).
- **Here I Stand**: Diplomatic track; secret deal-making during Diplomacy Phase.

**Commonality:** Common in CDGs and strategic games. Rare in pure operational/tactical games.

**Data model implications:**

- Named tracks with integer values, min/max bounds, and threshold effects.
- Multiple tracks per game (political, diplomatic, morale, etc.).
- Track changes triggered by events, card plays, combat results, or turn-based shifts.
- Tracks may be per-side, per-faction, per-region, or global.

---

### 3.10 Random Events

**How it works:** A random events table introduces unpredictable occurrences -- weather changes,
political shifts, supply windfalls, partisan activity, equipment failures, etc. Triggered by die
roll at start of turn or by card play.

**Commonality:** Common.

**Data model implications:**

- Event table: die_roll -> event_description + event_effects.
- Events need the same effect scripting system as card events.
- Events may be one-time or recurring.

---

### 3.11 Combined Arms Bonuses

**How it works:** Attacking with a mix of unit types (e.g., infantry + armor + artillery together)
provides a combat bonus, reflecting the real military advantage of combined arms.

**Implementation:**

- Column shift right when armor and infantry attack together (Bitter Woods: 1 column shift).
- DRM bonus for combined arms (GOSS: 5 DRM for armor/AT presence).
- Specific unit type combinations required to qualify.

**Commonality:** Common (most WWII operational games).

**Data model implications:**

- Combat modifier rules that check unit_type composition of attacking force.
- Combinatorial checks: presence of unit_type_A AND unit_type_B in same attack -> bonus.

---

### 3.12 Retreat and Advance After Combat (Complex Variants)

**How it works:** After combat resolution, retreating units must move a specified number of hexes
away from the attacker. Advancing units may move into the vacated hex (and sometimes beyond).

**Complex Retreat Rules:**

- Retreat through friendly units causes disruption to both.
- Retreat through enemy ZOC causes additional step losses.
- Retreat blocked by impassable terrain or enemy units = elimination.
- Retreat causes additional morale checks.
- "Rout" as extended involuntary retreat.

**Complex Advance Rules:**

- Only certain unit types may advance (armor can advance further than infantry).
- Advance triggers breakthrough combat eligibility.
- Advance limited to N hexes.
- Advance into fortified hex is restricted.

**Commonality:** Very common (most wargames have retreat/advance; complexity varies enormously).

**Data model implications:**

- Retreat: distance (N hexes), direction constraints, ZOC interaction, stacking interaction, terrain
  restrictions, morale effects.
- Advance: distance (N hexes), eligible unit types, exploitation triggers.
- Post-combat procedure is a sequence of sub-steps that must be configurable per game.

---

### 3.13 Surrender and Prisoner Mechanics

**How it works:** Isolated or surrounded units may surrender rather than fight to the death.
Prisoners may need to be escorted to rear areas, consuming resources.

**Commonality:** Uncommon. Some strategic games (Unconditional Surrender!) include surrender
mechanics.

**Data model implications:**

- Surrender trigger: isolation + combat result, or morale failure.
- Prisoner entities: captured units that must be moved/guarded.
- VP awards for prisoners captured.

---

### 3.14 Engineering (Bridges, Demolition, Mine Clearing)

**How it works:** Engineer units can build bridges, destroy them, lay minefields, clear mines, and
improve fortifications.

**Engineering Actions:**

| Action            | Effect                                                         | Commonality |
| ----------------- | -------------------------------------------------------------- | ----------- |
| Bridge building   | Creates bridge across river hexside; enables crossing          | Common      |
| Bridge demolition | Destroys bridge; blocks crossing                               | Common      |
| Minefield laying  | Creates minefield in hex; costs enemy MPs and may cause losses | Uncommon    |
| Mine clearing     | Removes minefield from hex                                     | Uncommon    |
| Road building     | Creates road in roadless hex                                   | Rare        |
| Fortification     | Engineers build defensive positions faster                     | Common      |

**Commonality:** Uncommon to common (most operational games have bridge rules; full engineering is
less common).

**Data model implications:**

- Engineer units: construction_rate, capabilities (bridge, demolition, mines).
- Bridge as hexside feature: can be created/destroyed during play.
- Minefield as hex feature: movement cost penalty, combat modifier, clearance rules.
- These are hex/hexside modifications that occur during play (not just at setup).

---

### 3.15 Night Rules

**How it works:** Night turns have modified rules: reduced visibility, reduced movement, modified
combat, increased chance of getting lost.

**Typical Night Effects:**

- LOS reduced to 1-2 hexes.
- Movement allowance halved or modified.
- Combat modifiers favor defender.
- Air operations prohibited or severely restricted.
- Unit identification restrictions (increased fog of war).

**Commonality:** Uncommon to common (tactical games frequently include night; operational games
sometimes).

**Data model implications:**

- Day/night as a per-turn attribute that modifies many other mechanics.
- Night effects: overlay that adjusts TEC, CRT, LOS, air rules.

---

### 3.16 Amphibious and Airborne Operations

**How it works:** Special rules for units arriving by sea (amphibious) or air (airborne).

**Amphibious:**

- Requires "amphibious prep points" accumulated over turns before the invasion.
- Landing hexes must be designated in advance.
- Defender's combat value is modified by coastal fortifications and adjacent friendly hexes.
- Naval support provides combat bonuses.
- Weather affects naval losses.

**Airborne:**

- Paradrop into enemy-held or neutral hexes.
- Scatter rolls determine actual landing hex (may deviate from target).
- Dropped units are isolated until linked up with ground forces.
- Limited to light equipment (no armor, limited artillery).

**Commonality:** Common in WWII games featuring D-Day, Market Garden, Crete, etc.

**Data model implications:**

- Special movement types: amphibious_landing, airborne_drop.
- Prep point accumulation system (turns of preparation before execution).
- Scatter mechanic: target hex + random deviation.
- Special combat modifiers for landing/drop.

---

### 3.17 Nuclear Weapons in Tactical Games

**How it works:** In Cold War-era tactical/operational games, nuclear weapons may be available. They
cause massive destruction in a hex area but may trigger escalation consequences.

**Example games:** Tactics II (1958), NATO: The Next War in Europe, some modern professional
wargames.

**Commonality:** Rare.

**Data model implications:**

- Area-of-effect weapon: affects target hex plus N-ring neighbors.
- Escalation track: nuclear use may trigger political consequences.
- Terrain transformation: nuclear strike may change terrain (crater, radiation).

---

### 3.18 Electronic Warfare

**How it works:** Represents jamming, signals intelligence, cyber warfare. May degrade enemy
command/control, provide intelligence advantages, or create additional dummy counters.

**Commonality:** Rare in traditional hex games. Increasingly relevant in modern-era games.

**Data model implications:**

- EW as a special action/mission type.
- Effects: degrade enemy command radius, add dummy counters, reveal hidden units, modify initiative.

---

### 3.19 Asymmetric Warfare / Insurgency (COIN Mechanics)

**How it works:** The COIN (COunter-INsurgency) series by Volko Ruhnke models asymmetric
multi-faction conflicts. Each of 2-4 factions has unique operations, capabilities, and victory
conditions.

**COIN Core Mechanics:**

- **Faction-specific operations**: Government can "Train" and "Sweep"; Insurgent can "Rally" and
  "March."
- **Special Activities**: Each faction has unique special activities (e.g., Assassinate, Airstrike,
  Extort).
- **Population tracks**: Support/Opposition per region determines political control.
- **Guerrilla units**: Hidden; must be revealed ("activated") before they can be targeted.
- **Eligibility track**: Determines turn order and limits who can act each card.
- **Resource management**: Each faction has limited resources (money, troops).
- **Area control without hexes**: COIN uses areas/regions, not hex grids (but the mechanics are
  instructive for the tool).

**Example games:** Andean Abyss, Cuba Libre, A Distant Plain, Fire in the Lake, Gandhi, Pendragon.

**Commonality:** Uncommon (COIN is a specific series, but its mechanics influence broader wargame
design).

**Data model implications:**

- Multi-faction system: more than 2 sides with asymmetric capabilities.
- Region-based control with population loyalty dimensions.
- Hidden guerrilla units with reveal mechanics.
- Per-faction operation menus: different actions available to different factions.
- Eligibility/initiative system distinct from IGOUGO.

**Sources:** [COIN Series - Wikipedia](<https://en.wikipedia.org/wiki/COIN_(board_game)>),
[Interview with Volko Ruhnke Part I](https://theplayersaid.com/2016/08/22/interview-with-coin-series-creator-designer-volko-ruhnke-part-i/),
[The COIN Series: 10 Reasons it is Great](https://theplayersaid.com/2023/09/12/the-coin-series-from-gmt-games-10-reasons-it-is-great/)

---

## Area 4: Game System Architecture

How rule sets are structured -- critical for the tool's data model.

---

### 4.1 Rulebook Numbering Systems

**How it works:** Wargame rulebooks use hierarchical decimal numbering. Each major section gets a
whole number; subsections use decimal points.

**Standard format:**

```
1.0 Introduction
2.0 Sequence of Play
  2.1 Player Turn Overview
  2.2 Reinforcement Phase
    2.21 Reinforcement Placement
    2.22 Delayed Reinforcements
  2.3 Movement Phase
3.0 Terrain
4.0 Stacking
5.0 Movement
6.0 Combat
...
```

**SPI convention:** Used dense decimal numbering (1.0, 1.1, 1.11, 1.12, 1.2...). Became the industry
standard.

**GMT convention:** Similar hierarchical numbering. Series rules (shared across all games in a
series) are separate from Exclusive Rules (game-specific). Living Rules documents are updated
post-publication.

**Series vs. Exclusive Rules:** Many game systems (OCS, GCACW, GBoH, GBACW) have:

- **Series Rules**: Core mechanics shared across all games in the series (e.g., OCS v4.3).
- **Exclusive Rules**: Game-specific additions and modifications (e.g., Hube's Pocket exclusive
  rules).

**Commonality:** Universal convention.

**Data model implications:**

- Rules are hierarchically structured: section -> subsection -> paragraph.
- Must support series/exclusive rule layering (base rules + game-specific overrides).
- Rule references (cross-references between sections) should be linkable.
- Optional rules: tagged as optional with on/off toggles that modify game behavior.

---

### 4.2 Standard Rulebook Sections

**Typical section order in a hex wargame rulebook:**

| Section                      | Content                                       |
| ---------------------------- | --------------------------------------------- |
| 1.0 Introduction             | Game overview, components list, setup         |
| 2.0 Sequence of Play         | Complete turn structure                       |
| 3.0 Game Terms / Definitions | Key terms defined                             |
| 4.0 Terrain                  | Terrain types and effects                     |
| 5.0 Stacking                 | Stacking limits and rules                     |
| 6.0 Movement                 | Movement points, terrain costs, road movement |
| 7.0 Zones of Control         | ZOC mechanics                                 |
| 8.0 Combat                   | CRT, modifiers, advance/retreat               |
| 9.0 Supply                   | Supply tracing and effects                    |
| 10.0 Reinforcements          | Entry schedule                                |
| 11.0 Special Rules           | Game-specific chrome                          |
| 12.0+ Optional Rules         | Additional complexity layers                  |
| Scenario Book                | Separate booklet with scenario setups         |

**Commonality:** Very common pattern (exact numbering varies but conceptual order is remarkably
consistent across publishers and decades).

**Data model implications:**

- The tool should organize game definitions in a structure mirroring this natural order.
- Each "section" maps to a configurable subsystem in the tool.

---

### 4.3 Scenario Definition Structure

**How scenarios are defined:**

| Element                      | Description                                               |
| ---------------------------- | --------------------------------------------------------- |
| **Scenario name**            | Historical name/date                                      |
| **Map area**                 | Which portion of the map is used (may be subset)          |
| **Turn range**               | Start turn and end turn                                   |
| **Weather**                  | Fixed or variable weather schedule                        |
| **Initial deployment**       | Per-side list of (unit, hex) placements                   |
| **Reinforcement schedule**   | Per-turn list of (unit, entry_hex, conditions)            |
| **Withdrawal schedule**      | Per-turn list of units removed from play                  |
| **Special scenario rules**   | Rule modifications specific to this scenario              |
| **Victory conditions**       | Per-side conditions, VP schedule, sudden death conditions |
| **Optional rules in effect** | Which optional rules are active                           |

**Commonality:** Universal for any game with multiple scenarios.

**Data model implications:**

- Scenario is a first-class entity that references: map, unit_list, deployment,
  reinforcement_schedule, withdrawal_schedule, victory_conditions, special_rules, optional_rules.
- Must support scenario variants (what-if modifications to historical scenarios).
- Deployment is a mapping: unit_id -> hex_id (or zone).

---

### 4.4 Orders of Battle (OOB) Structure

**How it works:** The OOB defines all units available in a game, their characteristics,
organizational relationships, and historical identities.

**OOB Structure:**

```
Army Group South
  6th Army
    51st Corps
      79th Infantry Division (3-3-5, 2 steps)
        79/1 Infantry Regiment (2-2-4, 1 step)
        79/2 Infantry Regiment (2-2-4, 1 step)
      ...
  1st Panzer Army
    3rd Panzer Corps
      14th Panzer Division (7-5-8, 2 steps)
      ...
```

**OOB Elements per unit:**

- Historical designation
- Unit type (NATO symbol)
- Unit size
- Parent formation (chain of command)
- Combat values (ATK/DEF/MOV and any other game-specific attributes)
- Steps / strength levels
- Setup hex (if in initial deployment) or reinforcement turn
- Special abilities or designations

**Data model implications:**

- OOB is a tree structure: army_group -> army -> corps -> division -> regiment -> battalion.
- Each node has: id, name, type, size, parent, attributes[], scenario_placement.
- Must support varying depths of hierarchy per game.
- Historical research often produces conflicting OOBs; must support sourcing/notes.

**Sources:**
[From Archive to OOB (WDS)](https://wargameds.com/blogs/news/from-archive-to-oob-part-i),
[Researching an Order of Battle (WDS)](https://wargameds.com/blogs/news/researching-an-order-of-battle-getting-started)

---

### 4.5 Combat Results Table Architecture

**Detailed CRT structure:**

```
Odds Column:  1:3  1:2  1:1  2:1  3:1  4:1  5:1  6:1+
Die Roll:
  1           AE   AE   AR   NE   DR   DR   DE   DE
  2           AE   AR   AR   NE   DR   DRL  DE   DE
  3           AR   AR   NE   DR   DR   DE   DE   DE
  4           AR   NE   NE   DR   DRL  DE   DE   DE
  5           NE   NE   DR   DRL  DE   DE   DE   DE
  6           NE   DR   DR   DE   DE   DE   DE   DE
```

**Die Roll Modifiers (DRM) vs. Column Shifts:**

- **DRM**: Add/subtract from die roll before looking up result. +1 DRM = add 1 to die.
- **Column shift**: Move left (worse for attacker) or right (better for attacker) N columns on CRT.
- Some games use both simultaneously.
- Column shifts and DRM are NOT equivalent: column shifts change the entire probability
  distribution; DRM shifts the result within a column.

**Commonality:** Universal CRT architecture (details vary per game).

**Data model implications:**

- CRT as a data table: list of columns (each with an odds_value or diff_value), list of rows (each
  with a die_value), and a 2D grid of result codes.
- Modifier pipeline: collect all applicable modifiers -> separate into DRM and column shifts ->
  apply shifts to select column -> apply DRM to die roll -> look up result.
- Result codes: game-defined enum; each code maps to a procedure (retreat N hexes, lose N steps,
  etc.).

---

### 4.6 Terrain Effects Chart Architecture

**Detailed TEC structure:**

```
Terrain Type | Movement Cost        | Combat Modifier        | Other
             | Inf | Mech | Horse  | ATK mod | DEF mod       | LOS | Supply | Stack
-------------|------|------|--------|---------|---------------|-----|--------|------
Clear        |  1   |  1   |   1    |   --    |    --         | Yes |  Yes   |  --
Woods        |  2   |  3   |   2    |   --    | +1 col shift  | No  |  Yes   |  --
City         |  1   |  2   |   1    |   --    | x2 defense    | No  |  Yes   |  +1
Mountain     |  3   | Proh |   3    |   --    | x2 defense    | Blk |  No*   |  -1
River (side) | +1   | +2   |  +1    | Half atk|    --         | --  |  --    |  --
Road (side)  | All=1| All=1| All=1  |   --    |    --         | --  |  Yes   |  --
```

**Data model implications:**

- TEC is a 2D lookup: terrain_type x (unit_type, effect_category) -> value.
- Movement costs: per unit movement_class (foot, tracked, wheeled, horse, etc.).
- Combat modifiers: may be column shifts, DRM, strength multipliers, or strength divisors.
- Other effects: LOS blocking, supply tracing, stacking modification.
- Hexside terrain has its own TEC entries (additive costs for crossing).

---

### 4.7 Turn Record and Reinforcement Track

**How it works:** A Turn Record Track (TRT) numbers each game turn, often with additional
information printed on each turn's box.

**TRT Information per Turn:**

- Turn number and date (e.g., "Turn 3: June 8, 1944")
- Weather determination (automatic or die roll required)
- Reinforcement arrivals (which units arrive)
- Withdrawals (which units depart)
- Special events (scenario-specific occurrences)
- Victory check timing
- Supply state changes

**Data model implications:**

- Turn record: ordered list of turn entries, each containing: turn_number, date,
  weather_instruction, reinforcements[], withdrawals[], events[], special_rules[].
- This is essentially a timeline of game events keyed to turn number.

---

## Area 5: Digital Implementation Considerations

Mechanics that pose specific challenges when implemented digitally.

---

### 5.1 Line of Sight (LOS) Calculations

**The challenge:** Determining whether one hex can "see" another requires tracing a line between hex
centers and checking for blocking terrain along the path. This is computationally simple but
geometrically tricky on hex grids.

**Approaches:**

- **Center-to-center ray**: Draw a line between hex centers. If any intervening hex contains
  blocking terrain (woods, buildings) at sufficient elevation, LOS is blocked.
- **Elevation-based**: Compare vertical angles. If obstacle's elevation angle from observer exceeds
  target's elevation angle, LOS is blocked.
- **Hex-spine ambiguity**: When the LOS line runs exactly along a hex spine (between two hexes),
  rules vary on which hex to check.
- **Blind hexes**: A hex directly behind a ridge or obstacle may be in a "dead zone" where LOS is
  blocked even though the hex itself is clear.
- **Dominant hills**: Elevated terrain that can see over intervening obstacles.

**Data model implications:**

- LOS algorithm: hex-center ray tracing with terrain/elevation checks.
- Per-hex: elevation (integer), los_blocking (bool per terrain type).
- Must handle hex-spine edge cases (consistent tie-breaking rule).
- Needs to be fast: called frequently for fire/combat eligibility checks.

**Sources:** [Line of Sight and Wargames](http://www.simmonsgames.com/design/LineOfSight.html),
[BGG: Hex-grid LOS Math](https://boardgamegeek.com/thread/403761/hex-grid-los-math-general-solution)

---

### 5.2 Hidden Information Management

**The challenge:** Board games struggle with hidden info (face-down counters, private rosters).
Digital implementation makes this trivial but requires a per-player visibility model.

**Requirements:**

- Each hex's contents must have per-player visibility states.
- Fog of war must be computable: what has each player ever seen?
- Simultaneous revelation on contact.
- Dummy units must be indistinguishable from real ones to the opponent's view.

**Data model implications:**

- Visibility layer: per-player map overlay tracking (never_seen, previously_seen,
  currently_visible).
- Per-unit: visible_to_player[] array.
- Reveal events: triggered by LOS, adjacency, or specific actions.

---

### 5.3 Complex Stacking and Counter Management

**The challenge:** Physical games can have 10+ counters in a hex (units, markers, fortifications,
supply). Digital must render this clearly.

**Issues:**

- Face-up vs. face-down counters in same stack.
- Markers (DG, reserve, supply, fortification) mixed with unit counters.
- Counter overflow: physically more counters than fit in a hex.
- Stack examination: player needs to see all counters in a stack.

**Data model implications:**

- Hex contents: ordered list of entities (units, markers, features).
- Display layering: which counters are visible and in what order.
- Stack summary: aggregate display (total attack, total defense, top unit info).

---

### 5.4 Multi-Hex Units

**The challenge:** Some games (especially naval) have units that span multiple hexes. Ship counters
may be 2-3 hexes long. Formation templates may define multi-hex positions.

**Issues:**

- Facing and rotation: which direction does a multi-hex unit face?
- Flanks and rear: defined differently when unit spans multiple hexes.
- Movement: entire multi-hex unit moves as one, paying worst-case terrain cost.

**Data model implications:**

- Unit footprint: set of hex offsets from a center hex.
- Facing: orientation (0-5 for hex directions).
- Movement: all hexes in footprint must be passable.

---

### 5.5 Off-Map Holding Boxes

**The challenge:** Many games have off-map areas (strategic reserve, dead pile, reorganization pool,
air display) that hold counters not currently on the map.

**Off-map box types:**

| Box                        | Purpose                                                  |
| -------------------------- | -------------------------------------------------------- |
| **Dead pile / Eliminated** | Destroyed units; may be eligible for rebuild             |
| **Reinforcement pool**     | Units waiting to enter as reinforcements                 |
| **Strategic reserve**      | Units temporarily off-map, can enter at designated hexes |
| **Reorganization pool**    | Eliminated units undergoing reconstruction               |
| **Air display**            | Air units not on the map; assigned to missions           |
| **Naval holding box**      | Naval units in port or at sea                            |
| **Replacement pool**       | Abstract replacement points                              |

**Data model implications:**

- Off-map locations are first-class entities with their own stacking and access rules.
- Units can transition between map hexes and off-map boxes.
- Each box type has rules for entry, exit, and capacity.

---

### 5.6 Strategic vs. Tactical Movement Modes

**The challenge:** Units may switch between movement modes with different capabilities and
restrictions.

**OCS Movement Modes:**

| Mode                  | MA      | Restrictions                                      |
| --------------------- | ------- | ------------------------------------------------- |
| **Move**              | Normal  | Standard movement; can enter ZOC                  |
| **Strategic (Strat)** | Double+ | Road-only; cannot enter enemy ZOC; cannot attack  |
| **Combat**            | Varies  | Prepared for combat; not affected by interdiction |
| **Exploitation**      | Half    | Only units awarded exploitation mode; can attack  |
| **Reserve**           | Quarter | Held back; can release during enemy reaction      |

**Data model implications:**

- Unit modes as a state machine: mode transitions have prerequisites and effects.
- Each mode modifies: MA, ZOC interaction, combat eligibility, vulnerability.
- Mode markers: tracked per unit per turn.

---

### 5.7 Complex Modifier Stacking

**The challenge:** A single combat may involve 10+ modifiers from different sources. Digital
implementation must correctly accumulate, cap, and apply these.

**Modifier pipeline example:**

1. Base attacker strength: sum of all attacking units' ATK values.
2. Terrain modifier: defender in city = halve attacker (or double defender).
3. Supply modifier: attacker out of supply = halve attack.
4. Weather modifier: mud = -1 column shift.
5. Combined arms: armor + infantry present = +1 column shift.
6. Flanking: attacking from 3 hexsides = +1 column shift.
7. Fortification: defender entrenched = -2 column shifts.
8. Leader: attacker in command = +1 DRM.
9. Unit quality: elite attacker = +1 DRM; green defender = -1 DRM.
10. Air support: CAS = +1 column shift.
11. Cross-river: attacking across river = halve attacker.

**Net column shifts and DRM are then applied to select CRT column and modify die roll.**

**Data model implications:**

- Modifiers must be collected from multiple sources (terrain, supply, weather, unit properties,
  special rules).
- Each modifier has: source, type (column_shift, drm, strength_multiplier, strength_halving),
  magnitude, applicability conditions.
- Modifier accumulation rules: are there caps? Do certain modifiers cancel others?
- Must be transparent: show the player exactly which modifiers are in effect and why.

---

## Area 6: Evolution of the Genre

How mechanics have evolved over time, and key designer contributions.

---

### 6.1 First Generation (1953-1969): Foundations

**Key characteristics:**

- Simple CRTs (often 1d6, 5-6 columns).
- Rigid ZOC (must stop, must attack).
- Basic terrain (clear, woods, mountain, river).
- Fixed turn structure (move then fight).
- No supply rules (or very simple).
- Unit counters with ATK-DEF-MOV only.

**Key games and innovations:**

- **Tactics (1954)** by Charles S. Roberts: First commercial board wargame. Established hex grid,
  CRT, and unit counters.
- **Gettysburg (1958)**: First historical battle simulation.
- **Afrika Korps (1964)**: Introduced supply as a game mechanic.
- **Jutland (1967)** by Jim Dunnigan: Naval warfare; search and contact mechanics.
- **1914 (1968)** by Dunnigan: WWI operational.

**Design tool relevance:** These games establish the minimum viable feature set. A design tool must
handle all first-generation mechanics as its baseline.

---

### 6.2 Second Generation (1969-1983): The SPI Era

**Key characteristics:**

- Explosion of complexity and detail.
- SPI published hundreds of games, many experimental.
- Introduction of: supply rules, step reduction, multiple CRT types, artillery, ZOC variants,
  stacking points, hidden units.
- Game systems (series rules) introduced to reduce per-game overhead.
- Extensive use of charts and tables.
- "Monster games" with thousands of counters and multiple maps.

**Key games and innovations:**

- **PanzerBlitz (1970)** by Dunnigan: Genre-defining tactical game. Sold 300,000+ copies. Introduced
  opportunity fire, unit-level detail.
- **War in Europe (1976)**: Monster game; multiple maps; full European theater.
- **Squad Leader (1977)**: Tactical game with programmed instruction rulebook.
- **The Russian Campaign (1974)**: Elegant operational design; weather and supply.
- **Third Reich (1974)**: Strategic WWII; production and diplomacy.

**Jim Dunnigan's contributions:**

- Founded SPI (1969) with Redmond Simonsen.
- Designed over 100 games.
- Codified wargame design methodology in "The Complete Wargames Handbook."
- Philosophy: "Build your own game on the work of others -- nobody owns the copyright to game
  mechanics."

**Richard Berg's contributions:**

- Designed 140+ games.
- Created the Great Battles of History series (tactical ancient/medieval combat).
- Innovations: formation cohesion, leader-driven activation, momentum/initiative mechanics, troop
  quality as primary combat factor.

**Design tool relevance:** Second-generation games require the tool to handle arbitrary complexity:
multi-step counters, multiple CRT types, elaborate supply rules, and large-scale operations.

---

### 6.3 Third Generation (1983-2000): Diversification

**Key characteristics:**

- Card-driven games (CDGs) introduced.
- Chit-pull activation systems gain prominence.
- Movement away from "more detail = more realism."
- Rise of "designer's games" (individual design visions over generic systems).
- Block games (Columbia Games).
- GCACW: Variable-length turns with initiative rolls.

**Key games and innovations:**

- **We The People (1994)** by Mark Herman: First CDG. Card-based combat replaces CRT. Revolutionized
  wargame design.
- **Hannibal: Rome vs. Carthage (1996)**: Dual-use strategy cards.
- **Paths of Glory (1999)** by Ted Raicer: CDG on hex map with traditional combat.
- **Advanced Squad Leader (1985)**: Culmination of tactical complexity (~200 tables).
- **Great Battles of History series (1992+)** by Berg/Herman: Ancient warfare; cohesion-based
  combat.
- **OCS (1992+)** by Dean Essig: Physical supply system; unit modes; exploitation.

**Mark Herman's contributions:**

- Invented the Card-Driven Game mechanic (1994).
- Designed Empire of the Sun (first CDG integrated with hex wargame).
- Philosophy: bridging narrative/event-driven gameplay with traditional hex mechanics.

**Dean Essig's contributions:**

- Created OCS series (definitive operational supply system).
- Created TCS (Tactical Combat Series: chit-pull tactical).
- Innovations: physical supply tokens, unit modes, exploitation as earned reward, reaction phases.

**Design tool relevance:** Third-generation games require the tool to handle cards as first-class
game objects, non-IGOUGO activation, and variable turn structures.

---

### 6.4 Fourth Generation (2000-Present): Modern Synthesis

**Key characteristics:**

- Hybrid mechanics combining CDG with hex-and-counter.
- COIN series: multi-faction asymmetric design.
- Return to elegant, playable designs with rich decision spaces.
- "Euro-wargame" crossover (simpler mechanics, deeper strategy).
- ZOC Bond system (Simonitch).
- Living rules (post-publication updates).
- GMT's P500 pre-order system enabling niche designs.

**Key games and innovations:**

- **Twilight Struggle (2005)**: CDG for Cold War political influence. Most-rated wargame on BGG.
- **COIN series (2012+)** by Volko Ruhnke: Multi-faction asymmetric insurgency. Deterministic
  combat. Population as center of gravity.
- **Normandy '44 (2010)** by Mark Simonitch: ZOC Bond system. Elegant operational WWII.
- **Combat Commander (2006)**: Card-driven tactical; no dice (cards determine randomness). Random
  events per card.
- **Undaunted series (2019)**: Deck-building meets tactical wargame.

**Mark Simonitch's contributions:**

- Created ZOC Bond system (ZOC + line between adjacent friendly units blocks enemy passage, supply,
  and retreat).
- Graphic design innovations (clear counter layouts, map aesthetics).
- Series of acclaimed WWII operational games (France '40, Holland '44, Ardennes '44, etc.).

**Volko Ruhnke's contributions:**

- Created COIN system for modeling asymmetric multi-faction conflict.
- Deterministic combat (no dice in many COIN games).
- Population loyalty as central mechanic.
- Cross-pollinated Eurogame and wargame design traditions.
- Philosophy: "Innovation is ideas being combined in new ways to new purposes."

**Design tool relevance:** Fourth-generation games require the tool to be maximally flexible --
supporting multi-faction play, deterministic combat, political tracks, population dynamics, and
hybrid mechanics that don't fit cleanly into traditional categories.

**Sources:** [Avalon Hill - Wikipedia](https://en.wikipedia.org/wiki/Avalon_Hill),
[SPI - Wikipedia](https://en.wikipedia.org/wiki/Simulations_Publications,_Inc.),
[Jim Dunnigan - Wikipedia](https://en.wikipedia.org/wiki/Jim_Dunnigan),
[Richard Berg - Wikipedia](https://en.wikipedia.org/wiki/Richard_Berg),
[Mark Simonitch - Wikipedia](https://en.wikipedia.org/wiki/Mark_Simonitch),
[The Complete Wargames Handbook (PDF)](https://www.professionalwargaming.co.uk/Complete-Wargames-Handbook-Dunnigan.pdf),
[Two Traditions of Wargame Design](https://hollandspiele.com/blogs/hollandazed-thoughts-ideas-and-miscellany/brief-thoughts-on-two-traditions-of-wargame-design)

---

## Summary: Data Model Implications for the Design Tool

Based on this survey, the tool must support these core entities and systems:

### Core Entities

1. **Hex** - coordinates, terrain, elevation, features, hexside terrain, fortification level
2. **Unit** - extensible attributes, steps, modes/states, formation hierarchy, visibility
3. **Map** - collection of hexes, off-map boxes, hex labels, scale metadata
4. **Scenario** - deployment, reinforcement/withdrawal schedules, victory conditions, special rules
5. **Player/Faction** - sides, resources, victory tracks, turn order

### Core Systems (Configurable Per Game)

1. **Movement System** - MP-based with terrain costs, multiple modes, road/rail/sea
2. **Combat System** - CRT (odds or differential), fire tables, card combat, modifiers
3. **ZOC System** - rigid/semi-rigid/fluid/bond, configurable effects
4. **Supply System** - trace/physical/HQ-radius, supply states, consumption
5. **Command System** - HQ radius, chain of command, activation
6. **Morale System** - states, checks, rally, cascading
7. **Turn Structure** - phases, IGOUGO/chit-pull/impulse, reaction windows
8. **Stacking System** - limits by count/steps/points, terrain modifiers
9. **Weather System** - per-turn/per-zone states, effects on other systems
10. **Card System** - dual-use cards, hand management, events
11. **Victory System** - hex control, VP, tracks, asymmetric, sudden death
12. **Visibility System** - hidden/concealed/revealed, per-player fog of war

### Key Architectural Principles

- **Composability**: Each mechanic is an independent module that can be included or excluded.
- **Extensibility**: Unit attributes, terrain types, CRT result codes, and modifier sources must be
  user-defined, not hardcoded.
- **Series/Game Layering**: Base rules (series) with per-game overrides (exclusive rules).
- **Scale Agnosticism**: The tool should not enforce scale-specific mechanic restrictions.
- **Effect Scripting**: Card events, random events, and special rules need a way to encode arbitrary
  game effects.
- **Modifier Pipeline**: Combat (and other resolution) must support transparent accumulation and
  application of modifiers from many sources.
