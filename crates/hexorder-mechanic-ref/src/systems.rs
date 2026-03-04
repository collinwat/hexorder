//! Catalog content population.
//!
//! Entries drawn from the Hex Wargame Mechanics Survey (wiki) organized
//! by the six areas of the Engelstein taxonomy.

use super::components::{
    MechanicCatalog, MechanicCategory, MechanicEntry, ScaffoldAction, ScaffoldRecipe,
    TemplateAvailability,
};

/// Helper to reduce boilerplate when constructing entries.
fn entry(
    name: &str,
    category: MechanicCategory,
    description: &str,
    examples: &[&str],
    considerations: &str,
) -> MechanicEntry {
    MechanicEntry {
        name: name.to_string(),
        category,
        description: description.to_string(),
        example_games: examples.iter().map(|s| (*s).to_string()).collect(),
        design_considerations: considerations.to_string(),
        template: TemplateAvailability::None,
    }
}

/// Helper for entries that have a scaffolding template.
#[allow(clippy::too_many_arguments)]
fn templated_entry(
    name: &str,
    category: MechanicCategory,
    description: &str,
    examples: &[&str],
    considerations: &str,
    template_id: &str,
    preview: &str,
) -> MechanicEntry {
    MechanicEntry {
        name: name.to_string(),
        category,
        description: description.to_string(),
        example_games: examples.iter().map(|s| (*s).to_string()).collect(),
        design_considerations: considerations.to_string(),
        template: TemplateAvailability::Available {
            template_id: template_id.to_string(),
            preview: preview.to_string(),
        },
    }
}

/// Build the full mechanic reference catalog from the Hex Wargame Mechanics Survey.
pub fn create_catalog() -> MechanicCatalog {
    let mut entries = Vec::with_capacity(56);

    // -----------------------------------------------------------------------
    // Area 1: Core Universal Mechanics (1.1 – 1.9)
    // -----------------------------------------------------------------------
    let core = MechanicCategory::CoreUniversal;

    entries.push(entry(
        "Hex Grid Systems",
        core,
        "The game map is overlaid with a hexagonal grid where each hex represents a geographic \
         area at a specific scale. Hexes provide six equidistant neighbors, enabling more natural \
         movement and facing than square grids.",
        &["Panzerblitz", "Squad Leader", "Gettysburg"],
        "Must support offset, axial, and cube coordinate systems with conversion. Requires \
         adjacency queries, distance calculations, pathfinding, and LOS ray tracing.",
    ));

    entries.push(templated_entry(
        "Movement Systems",
        core,
        "Each unit has a Movement Allowance of movement points per turn. Each hex costs MPs to \
         enter based on terrain and unit type. Sub-mechanics include road movement, minimum move, \
         strategic movement, and hexside costs.",
        &["Panzerblitz", "OCS", "Third Reich"],
        "Units need movement_allowance and movement_type. A Terrain Effects Chart maps terrain \
         type x unit movement type to MP cost. Must model hexside terrain distinct from hex \
         terrain.",
        "movement_points",
        "Creates movement_allowance property on tokens and movement_cost on cells.",
    ));

    entries.push(templated_entry(
        "Combat Resolution Systems",
        core,
        "Attacking units designate targets, combat strengths are compared, modifiers applied, \
         dice rolled, and results read from a CRT. CRT types include odds-ratio, differential, \
         fire table, card-driven, and hybrid approaches.",
        &[
            "Panzerblitz",
            "Third Reich",
            "Advanced Squad Leader",
            "We The People",
            "Paths of Glory",
        ],
        "CRT is a first-class data structure with game-defined result codes. Must support ratio \
         and differential calculation, modifier pipelines for column shifts and DRMs.",
        "crt_combat",
        "Creates a standard odds-based CRT with columns, rows, outcomes, and terrain modifiers.",
    ));

    entries.push(entry(
        "Turn Structure and Phasing",
        core,
        "The game proceeds in discrete turns divided into phases in a fixed Sequence of Play. \
         The most common is IGOUGO where one player completes all phases then the other does the \
         same. Variants include alternating activation and impulse systems.",
        &["Combat Commander", "Paths of Glory", "Diplomacy"],
        "Turn structure is a tree (Turn -> Player Turn -> Phase). Must support fixed-order and \
         random/conditional phase sequences, activation pools, and reaction windows.",
    ));

    entries.push(templated_entry(
        "Zones of Control",
        core,
        "Most combat units project a Zone of Control into surrounding hexes representing \
         observation, threat, and influence. ZOC types include rigid, semi-rigid, fluid, locking, \
         and terrain-negated.",
        &["Normandy '44", "Ardennes '44", "Ukraine '43", "France '40"],
        "ZOC is a unit property with configurable type and effects (movement cost, stop/no-stop, \
         supply blocking, retreat interaction).",
        "zones_of_control",
        "Creates ZOC type enum and properties for ZOC projection and effects.",
    ));

    entries.push(templated_entry(
        "Stacking Rules",
        core,
        "Stacking limits restrict how many units can occupy a single hex, expressed as unit \
         count, step count, or stacking points. Checked at end of movement; markers often exempt.",
        &["OCS", "Columbia block games"],
        "Each game defines stacking metric, limit, and exceptions. Terrain may modify limits. \
         Transient overstacking during movement must be handled.",
        "stacking",
        "Creates stacking_limit on cells and stacking_value on tokens.",
    ));

    entries.push(templated_entry(
        "Terrain System",
        core,
        "Each hex contains a primary terrain type and hexsides may have features. Terrain affects \
         movement cost, combat modifiers, LOS, supply tracing, and stacking. The TEC is the \
         central reference matrix.",
        &["Panzerblitz", "Squad Leader"],
        "TEC is a core data table of terrain type x effect category x unit type. Terrain types \
         must be definable per game, not a fixed set.",
        "terrain_effects",
        "Creates terrain type enum and defense/movement modifier properties on cells.",
    ));

    entries.push(entry(
        "Unit Attributes and Counter Design",
        core,
        "Each unit counter encodes capabilities as numeric values and NATO military symbology. \
         Standard attributes include attack/defense strength, movement allowance, unit type, \
         size, and formation. Step reduction via flip, replacement, or block rotation.",
        &["OCS", "Advanced Squad Leader", "Columbia Games", "GBoH"],
        "Unit is the central entity with extensible attributes. Must support unit templates, \
         current vs. full-strength values, organizational hierarchy, and custom attributes.",
    ));

    entries.push(entry(
        "Victory Conditions",
        core,
        "The game defines how a winner is determined through hex control, victory points, sudden \
         death, graduated victory, territorial control, attrition, or asymmetric per-faction \
         conditions.",
        &["COIN series", "Twilight Struggle"],
        "Victory conditions are per-scenario with support for hex control checks, VP tallies, \
         threshold comparisons, and graduated levels.",
    ));

    // -----------------------------------------------------------------------
    // Area 2: Advanced/Common Mechanics (2.1 – 2.10)
    // -----------------------------------------------------------------------
    let advanced = MechanicCategory::AdvancedCommon;

    entries.push(templated_entry(
        "Supply and Logistics",
        advanced,
        "Units must trace a supply line to a supply source; being out of supply degrades \
         effectiveness. Variants include binary trace, graduated supply, physical supply tokens \
         (OCS), and HQ-radius supply.",
        &["OCS", "Third Reich", "Case Blue"],
        "Need supply sources as tagged hexes, pathfinding for supply tracing, and supply state \
         effects on unit capabilities.",
        "supply",
        "Creates supply_source on cells, supply_range and in_supply on tokens.",
    ));

    entries.push(entry(
        "Command and Control",
        advanced,
        "Units must be in command via proximity to an HQ to operate at full effectiveness. \
         Mechanics include command radius, chain of command, out-of-command penalties, and \
         activation limits.",
        &["GCACW", "GBoH", "GOSS"],
        "HQ units need command radius and capacity. Command chain is a tree structure. In-command \
         is a computed status modifying unit capabilities.",
    ));

    entries.push(entry(
        "Morale and Cohesion",
        advanced,
        "Units have a morale value that degrades under combat stress. Progression is typically \
         Normal -> Disrupted -> Demoralized -> Routed -> Eliminated. Rally requires proximity to \
         leaders.",
        &["Advanced Squad Leader", "GBoH", "OCS"],
        "Units need morale_value and current state as a state machine with configurable \
         transitions, triggers, and rally conditions.",
    ));

    entries.push(entry(
        "Weather Effects",
        advanced,
        "Weather changes across turns affecting movement costs, combat modifiers, and air \
         operations. Determined randomly via weather table at turn start. Dimensions include \
         ground conditions and air conditions.",
        &["War in the East 2", "OCS"],
        "Weather state can be per-turn or per-zone. Need weather tables keyed to turn/season and \
         weather effects that modify TEC, supply range, and air availability.",
    ));

    entries.push(entry(
        "Fog of War / Limited Intelligence",
        advanced,
        "Players have incomplete information about enemy forces via face-down counters, dummy \
         counters, block games, double-blind play, or concealment markers.",
        &["Columbia Games series", "Advanced Squad Leader"],
        "Units need visibility state (hidden, concealed, revealed). Per-player visibility model \
         with reveal triggers (adjacency, combat, reconnaissance).",
    ));

    entries.push(entry(
        "Air Power",
        advanced,
        "Air units represent squadrons placed in missions rather than moving hex-by-hex. They \
         provide CAS, interdiction, air superiority, strategic bombing, and reconnaissance from \
         an off-map air display.",
        &["Empire of the Sun", "OCS", "Third Reich"],
        "Air units may live off-map. Missions are assignments to targets. Air availability is \
         pool-based (N air points per turn).",
    ));

    entries.push(entry(
        "Artillery and Indirect Fire",
        advanced,
        "Artillery units attack targets at range without adjacency via bombardment/barrage \
         tables. Mechanics include barrage strength, range, counterbattery, forward observers, \
         and ammo consumption.",
        &["OCS", "Advanced Squad Leader"],
        "Barrage table is separate from main CRT. Must check range, LOS, and spotter \
         requirements. Artillery integrates with ground combat as a modifier.",
    ));

    entries.push(entry(
        "Fortifications and Entrenchment",
        advanced,
        "Units build defensive positions over time gaining increasing defensive bonuses. \
         Engineer units build faster; terrain and supply affect construction rate.",
        &["War in the East 2", "OCS"],
        "Per-hex fortification level as integer with construction rules accumulating toward \
         thresholds. Fortification effects modify TEC combat modifiers.",
    ));

    entries.push(entry(
        "Replacements and Reinforcements",
        advanced,
        "New units enter per schedule, existing units rebuild via replacement points, and units \
         may be withdrawn on schedule. Replacement variants include points per turn and automatic \
         step recovery.",
        &["Most scenario-based wargames"],
        "Need reinforcement and withdrawal schedules, replacement pool as per-side resource, and \
         unit lifecycle states (on map, in reserve, eliminated, withdrawn).",
    ));

    entries.push(entry(
        "Scale Differences",
        advanced,
        "Wargames operate at different scales (tactical, operational, strategic) which \
         fundamentally affects which mechanics are relevant. Each scale emphasizes different unit \
         sizes, turn lengths, and key mechanics.",
        &["Advanced Squad Leader", "OCS", "War in Europe"],
        "The tool must be scale-agnostic, allowing free composition of mechanics rather than \
         enforcing scale-specific restrictions.",
    ));

    // -----------------------------------------------------------------------
    // Area 3: Bespoke/Unusual Mechanics (3.1 – 3.19)
    // -----------------------------------------------------------------------
    let bespoke = MechanicCategory::BespokeUnusual;

    entries.push(entry(
        "Chit-Pull Activation Systems",
        bespoke,
        "Formation chits drawn randomly from a cup determine which formation activates next, \
         replacing alternating full player turns. Variants include formation activation, quality-\
         based multiple chits, and end-of-turn chits.",
        &["Combat Commander", "Ardennes '44", "The Gamers TCS"],
        "Activation pool of chit definitions with draw probabilities. Must support multiple \
         draws per turn and variable turn length.",
    ));

    entries.push(entry(
        "Card-Driven Game Mechanics",
        bespoke,
        "Players hold cards usable either for printed events or operations value, creating \
         tension between powerful events and ops points. Introduced by Mark Herman in We The \
         People (1994).",
        &[
            "We The People",
            "Hannibal: Rome vs. Carthage",
            "Paths of Glory",
            "Twilight Struggle",
        ],
        "Cards need ops value, event text, event effects, owning side, and removal flag. Event \
         effects require a scripting system to encode arbitrary card effects.",
    ));

    entries.push(entry(
        "Overrun Mechanics",
        bespoke,
        "During movement, a strong unit can overrun a weak enemy in its path, attacking on the \
         move without stopping. Resolved immediately using CRT at favorable odds.",
        &["OCS", "Stalingrad '42"],
        "Overrun is a movement-phase combat action requiring MP cost, minimum odds, and CRT \
         reference. Must interrupt normal movement to resolve combat mid-move.",
    ));

    entries.push(entry(
        "Exploitation and Breakthrough",
        bespoke,
        "After successful combat, certain units (especially armor) make additional moves and \
         attacks in a special exploitation phase, simulating blitzkrieg breakthrough.",
        &["OCS", "Stalingrad '42"],
        "Unit exploitation eligibility set by CRT result. Additional phase in sequence of play \
         with restricted eligibility.",
    ));

    entries.push(entry(
        "Reserve Commitment",
        bespoke,
        "Units in Reserve Mode are held back from regular operations but can be released during \
         the enemy's turn in response to enemy actions.",
        &["OCS"],
        "Unit mode state with capability restrictions. Reaction phase during opponent's turn for \
         reserve commitment.",
    ));

    entries.push(entry(
        "Reaction and Opportunity Fire",
        bespoke,
        "During enemy movement, defending units can interrupt to fire at moving enemy units. \
         Variants include defensive fire, opportunity fire, overwatch, and final protective fire.",
        &["Advanced Squad Leader", "Squad Leader", "Panzer"],
        "Interruption system with reaction eligibility (range, LOS, remaining shots). Needs shot \
         tracking and fire table resolution.",
    ));

    entries.push(entry(
        "Hidden Units and Dummy Counters",
        bespoke,
        "Real units mixed with dummy counters that opponents cannot distinguish until contact or \
         reconnaissance reveals them.",
        &["Rommel in the Desert", "Advanced Squad Leader"],
        "Dummy counter type with no combat value. Reveal triggers based on adjacency, LOS, \
         recon, or combat. Per-player visibility model.",
    ));

    entries.push(entry(
        "Variable Turn Length",
        bespoke,
        "Instead of a fixed turn count, the game might end randomly after a certain point via \
         random end checks or sudden death conditions.",
        &["Warhammer 40K", "GCACW"],
        "Turn end condition as per-turn check. Must support both fixed and variable-length games. \
         Victory evaluation must work at any turn.",
    ));

    entries.push(entry(
        "Political and Diplomatic Tracks",
        bespoke,
        "Abstract tracks represent non-military dimensions: political support, diplomatic \
         relations, war weariness, and national morale.",
        &[
            "Twilight Struggle",
            "COIN series",
            "Paths of Glory",
            "Here I Stand",
        ],
        "Named tracks with integer values, min/max bounds, and threshold effects. Changes \
         triggered by events, card plays, or combat results.",
    ));

    entries.push(entry(
        "Random Events",
        bespoke,
        "A random events table introduces unpredictable occurrences such as weather changes, \
         political shifts, supply windfalls, and partisan activity.",
        &["Many wargames"],
        "Event table maps die roll to event description and effects. Events need the same effect \
         scripting system as card events.",
    ));

    entries.push(entry(
        "Combined Arms Bonuses",
        bespoke,
        "Attacking with a mix of unit types (infantry + armor + artillery) provides combat \
         bonuses reflecting real military advantage, implemented as column shifts or DRM bonuses.",
        &["Bitter Woods", "GOSS"],
        "Combat modifier rules that check unit type composition of attacking force for specific \
         combinations triggering bonuses.",
    ));

    entries.push(entry(
        "Retreat and Advance After Combat",
        bespoke,
        "After combat, retreating units move away from attackers while advancing units move into \
         vacated hexes. Complex variants include ZOC losses, disruption, and exploitation \
         triggers.",
        &["Most wargames"],
        "Retreat needs configurable distance, ZOC interaction, and morale effects. Advance needs \
         eligible types and distance limits. Post-combat procedure must be configurable.",
    ));

    entries.push(entry(
        "Surrender and Prisoner Mechanics",
        bespoke,
        "Isolated or surrounded units may surrender rather than fight to the death. Prisoners \
         may need to be escorted to rear areas.",
        &["Unconditional Surrender!"],
        "Surrender triggers based on isolation plus combat result or morale failure. Prisoner \
         entities that must be moved and guarded.",
    ));

    entries.push(entry(
        "Engineering",
        bespoke,
        "Engineer units can build bridges, destroy them, lay minefields, clear mines, and \
         improve fortifications. These modify hex and hexside features during play.",
        &["Most operational games"],
        "Engineer units need construction rate and capabilities. Bridge and minefield as \
         creatable/destroyable hex or hexside features.",
    ));

    entries.push(entry(
        "Night Rules",
        bespoke,
        "Night turns have modified rules including reduced visibility, reduced movement, \
         defender-favoring combat modifiers, and restricted air operations.",
        &["Many tactical and operational games"],
        "Day/night as per-turn attribute modifying many mechanics. Night effects are an overlay \
         adjusting TEC, CRT, LOS, and air rules.",
    ));

    entries.push(entry(
        "Amphibious and Airborne Operations",
        bespoke,
        "Special rules for units arriving by sea (amphibious with prep points and designated \
         landing hexes) or air (airborne with paradrop, scatter, and isolation until link-up).",
        &["D-Day games", "Market Garden games"],
        "Special movement types for amphibious landing and airborne drop. Prep point accumulation \
         and scatter mechanics with random deviation.",
    ));

    entries.push(entry(
        "Nuclear Weapons",
        bespoke,
        "In Cold War-era games, nuclear weapons may be available causing massive area destruction \
         but potentially triggering escalation consequences.",
        &["Tactics II", "NATO: The Next War in Europe"],
        "Area-of-effect weapon affecting target hex plus ring. Escalation track for political \
         consequences.",
    ));

    entries.push(entry(
        "Electronic Warfare",
        bespoke,
        "Represents jamming, signals intelligence, and cyber warfare that may degrade enemy \
         command and control or provide intelligence advantages.",
        &["Modern-era tactical games"],
        "EW as a special mission type with effects on enemy command radius, dummy counter \
         creation, and initiative modification.",
    ));

    entries.push(entry(
        "Asymmetric Warfare / COIN",
        bespoke,
        "The COIN series models asymmetric multi-faction conflicts with faction-specific \
         operations, population support tracks, hidden guerrilla units, and eligibility tracks.",
        &[
            "Andean Abyss",
            "Cuba Libre",
            "A Distant Plain",
            "Fire in the Lake",
        ],
        "Multi-faction system with asymmetric capabilities. Region-based control with population \
         loyalty. Per-faction operation menus.",
    ));

    // -----------------------------------------------------------------------
    // Area 4: Game System Architecture (4.1 – 4.7)
    // -----------------------------------------------------------------------
    let architecture = MechanicCategory::GameSystemArchitecture;

    entries.push(entry(
        "Rulebook Numbering Systems",
        architecture,
        "Wargame rulebooks use hierarchical decimal numbering (SPI convention). Series rules are \
         shared across all games; exclusive rules are game-specific additions. Living rules are \
         updated post-publication.",
        &["SPI games", "GMT games", "OCS", "GCACW"],
        "Rules are hierarchically structured. Must support series/exclusive rule layering and \
         optional rule toggles.",
    ));

    entries.push(entry(
        "Standard Rulebook Sections",
        architecture,
        "Hex wargame rulebooks follow a consistent section order: Introduction, Sequence of \
         Play, Terms, Terrain, Stacking, Movement, ZOC, Combat, Supply, Reinforcements, Special \
         Rules, Scenarios.",
        &["Most published wargames"],
        "The tool should organize game definitions mirroring this natural order. Each section \
         maps to a configurable subsystem.",
    ));

    entries.push(entry(
        "Scenario Definition Structure",
        architecture,
        "Scenarios define map area, turn range, weather, initial deployment, reinforcement and \
         withdrawal schedules, special rules, and victory conditions.",
        &["Any multi-scenario game"],
        "Scenario is a first-class entity referencing map, units, deployment, schedules, and \
         victory conditions. Must support scenario variants.",
    ));

    entries.push(entry(
        "Orders of Battle Structure",
        architecture,
        "The OOB defines all units with characteristics, organizational relationships, and \
         historical identities as a hierarchical tree from army group down to battalion.",
        &["All operational/strategic wargames"],
        "OOB is a tree structure with varying depths. Each node has id, name, type, size, \
         parent, and attributes.",
    ));

    entries.push(entry(
        "Combat Results Table Architecture",
        architecture,
        "CRT is a 2D matrix with odds/differential columns and die roll rows. DRMs add to die \
         rolls; column shifts move along the axis. Both may apply simultaneously and are not \
         equivalent.",
        &["Universal CRT architecture"],
        "CRT as data table with modifier pipeline: collect modifiers, separate DRM from column \
         shifts, apply shifts, apply DRM, look up result.",
    ));

    entries.push(entry(
        "Terrain Effects Chart Architecture",
        architecture,
        "TEC is a matrix with terrain type rows and columns grouped into movement cost, combat \
         effect, and other effects. Movement costs vary by unit movement class.",
        &["Universal across hex wargames"],
        "TEC is a 2D lookup of terrain type x (unit type, effect category) to value. Hexside \
         terrain has its own additive TEC entries.",
    ));

    entries.push(entry(
        "Turn Record and Reinforcement Track",
        architecture,
        "A Turn Record Track numbers each game turn with per-turn information: date, weather, \
         reinforcement arrivals, withdrawals, special events, and victory checks.",
        &["Most scenario-based wargames"],
        "Turn record is an ordered list of entries containing turn number, date, weather, \
         reinforcements, withdrawals, and events.",
    ));

    // -----------------------------------------------------------------------
    // Area 5: Digital Implementation Considerations (5.1 – 5.7)
    // -----------------------------------------------------------------------
    let digital = MechanicCategory::DigitalImplementation;

    entries.push(entry(
        "Line of Sight Calculations",
        digital,
        "Determining whether one hex can see another requires tracing a line between hex centers \
         and checking for blocking terrain. Approaches include center-to-center ray and \
         elevation-based angle comparison.",
        &["All tactical games", "Many operational games"],
        "LOS algorithm needs hex-center ray tracing with terrain/elevation checks. Must handle \
         hex-spine edge cases with consistent tie-breaking.",
    ));

    entries.push(entry(
        "Hidden Information Management",
        digital,
        "Digital implementation requires a per-player visibility model tracking what each player \
         has seen and currently sees. Simultaneous revelation on contact and indistinguishable \
         dummy units are key requirements.",
        &["All fog of war games"],
        "Visibility layer per player tracking never-seen, previously-seen, and currently-visible. \
         Reveal events triggered by LOS, adjacency, or actions.",
    ));

    entries.push(entry(
        "Complex Stacking Display",
        digital,
        "Physical games can have 10+ counters per hex. Digital must render this clearly, handling \
         face-up vs face-down counters, markers mixed with units, and stack examination.",
        &["Complex operational games"],
        "Hex contents as ordered list of entities. Display layering for visibility. Stack summary \
         showing aggregate info.",
    ));

    entries.push(entry(
        "Multi-Hex Units",
        digital,
        "Some games (especially naval) have units spanning multiple hexes. Issues include facing, \
         flanks, and movement paying worst-case terrain cost across all occupied hexes.",
        &["Naval games with multi-hex ships"],
        "Unit footprint as set of hex offsets from center. Facing as orientation. Movement \
         requires all hexes in footprint to be passable.",
    ));

    entries.push(entry(
        "Off-Map Holding Boxes",
        digital,
        "Many games have off-map areas (dead pile, reinforcement pool, strategic reserve, air \
         display) holding counters not on the map.",
        &["Most operational/strategic games"],
        "Off-map locations are first-class entities with stacking and access rules. Units \
         transition between map hexes and off-map boxes.",
    ));

    entries.push(entry(
        "Movement Mode Management",
        digital,
        "Units switch between movement modes (Move, Strategic, Combat, Exploitation, Reserve) \
         with different MA values, ZOC interaction, and combat eligibility.",
        &["OCS"],
        "Unit modes as a state machine with prerequisites and effects for transitions. Each mode \
         modifies MA, ZOC interaction, and vulnerability.",
    ));

    entries.push(entry(
        "Complex Modifier Stacking",
        digital,
        "A single combat may involve 10+ modifiers from terrain, supply, weather, combined arms, \
         flanking, fortification, and more. Digital must correctly accumulate, cap, and display \
         these.",
        &["Most operational games"],
        "Modifiers collected from multiple sources with type (column shift, DRM, multiplier), \
         magnitude, and applicability conditions. Must support caps and transparent display.",
    ));

    // -----------------------------------------------------------------------
    // Area 6: Evolution of the Genre (6.1 – 6.4)
    // -----------------------------------------------------------------------
    let evolution = MechanicCategory::GenreEvolution;

    entries.push(entry(
        "First Generation (1953-1969)",
        evolution,
        "Charles S. Roberts established the hex grid, CRT, and unit counters with Tactics \
         (1954). Simple CRTs, rigid ZOC, basic terrain, fixed turns, no supply. Afrika Korps \
         (1964) introduced supply.",
        &["Tactics", "Gettysburg", "Afrika Korps", "Jutland", "1914"],
        "These games establish the minimum viable feature set. A design tool must handle all \
         first-generation mechanics as its baseline.",
    ));

    entries.push(entry(
        "Second Generation (1969-1983)",
        evolution,
        "Explosion of complexity with SPI publishing hundreds of games. Introduced step \
         reduction, multiple CRT types, artillery, ZOC variants, stacking points, hidden units, \
         and game series systems.",
        &[
            "PanzerBlitz",
            "War in Europe",
            "Squad Leader",
            "The Russian Campaign",
            "Third Reich",
        ],
        "Second-generation games require handling arbitrary complexity: multi-step counters, \
         multiple CRT types, and elaborate supply rules.",
    ));

    entries.push(entry(
        "Third Generation (1983-2000)",
        evolution,
        "Card-driven games introduced, chit-pull activation gains prominence, movement away from \
         'more detail = more realism,' rise of block games and variable-length turns.",
        &[
            "We The People",
            "Hannibal: Rome vs. Carthage",
            "Paths of Glory",
            "Advanced Squad Leader",
            "OCS",
        ],
        "Third-generation games require cards as first-class game objects, non-IGOUGO \
         activation, and variable turn structures.",
    ));

    entries.push(entry(
        "Fourth Generation (2000-Present)",
        evolution,
        "Hybrid CDG/hex mechanics, COIN multi-faction asymmetric design, return to elegant \
         playable designs, Euro-wargame crossover, and ZOC Bond system.",
        &[
            "Twilight Struggle",
            "COIN series",
            "Normandy '44",
            "Combat Commander",
            "Undaunted series",
        ],
        "Fourth-generation games require maximal flexibility: multi-faction play, political \
         tracks, population dynamics, and hybrid mechanics.",
    ));

    let templates = vec![
        crt_combat_template(),
        movement_points_template(),
        zones_of_control_template(),
        stacking_template(),
        terrain_effects_template(),
        supply_template(),
    ];

    MechanicCatalog { entries, templates }
}

// ---------------------------------------------------------------------------
// Scaffolding template definitions
// ---------------------------------------------------------------------------

fn crt_combat_template() -> ScaffoldRecipe {
    ScaffoldRecipe {
        template_id: "crt_combat".to_string(),
        description: "Standard odds-based CRT with 6 columns, 6 die rows, and terrain modifiers."
            .to_string(),
        actions: vec![
            // CRT columns (odds ratios)
            ScaffoldAction::AddCrtColumn {
                label: "1:2".to_string(),
                column_type: "odds_ratio".to_string(),
                threshold: 0.5,
            },
            ScaffoldAction::AddCrtColumn {
                label: "1:1".to_string(),
                column_type: "odds_ratio".to_string(),
                threshold: 1.0,
            },
            ScaffoldAction::AddCrtColumn {
                label: "2:1".to_string(),
                column_type: "odds_ratio".to_string(),
                threshold: 2.0,
            },
            ScaffoldAction::AddCrtColumn {
                label: "3:1".to_string(),
                column_type: "odds_ratio".to_string(),
                threshold: 3.0,
            },
            ScaffoldAction::AddCrtColumn {
                label: "4:1".to_string(),
                column_type: "odds_ratio".to_string(),
                threshold: 4.0,
            },
            ScaffoldAction::AddCrtColumn {
                label: "5:1".to_string(),
                column_type: "odds_ratio".to_string(),
                threshold: 5.0,
            },
            // Die roll rows (1d6)
            ScaffoldAction::AddCrtRow {
                label: "1".to_string(),
                die_min: 1,
                die_max: 1,
            },
            ScaffoldAction::AddCrtRow {
                label: "2".to_string(),
                die_min: 2,
                die_max: 2,
            },
            ScaffoldAction::AddCrtRow {
                label: "3".to_string(),
                die_min: 3,
                die_max: 3,
            },
            ScaffoldAction::AddCrtRow {
                label: "4".to_string(),
                die_min: 4,
                die_max: 4,
            },
            ScaffoldAction::AddCrtRow {
                label: "5".to_string(),
                die_min: 5,
                die_max: 5,
            },
            ScaffoldAction::AddCrtRow {
                label: "6".to_string(),
                die_min: 6,
                die_max: 6,
            },
            // Sample outcomes (row 0 = die 1, col 0 = 1:2)
            ScaffoldAction::SetCrtOutcome {
                row: 0,
                col: 0,
                label: "AE".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 0,
                col: 1,
                label: "AE".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 0,
                col: 2,
                label: "AR".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 0,
                col: 3,
                label: "NE".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 0,
                col: 4,
                label: "DR".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 0,
                col: 5,
                label: "DR".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 5,
                col: 0,
                label: "AR".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 5,
                col: 1,
                label: "NE".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 5,
                col: 2,
                label: "DR".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 5,
                col: 3,
                label: "DR".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 5,
                col: 4,
                label: "DE".to_string(),
            },
            ScaffoldAction::SetCrtOutcome {
                row: 5,
                col: 5,
                label: "DE".to_string(),
            },
            // Terrain defense modifier
            ScaffoldAction::AddCombatModifier {
                name: "Forest defense".to_string(),
                source: "defender_terrain".to_string(),
                shift: -1,
                priority: 10,
            },
            ScaffoldAction::AddCombatModifier {
                name: "City defense".to_string(),
                source: "defender_terrain".to_string(),
                shift: -2,
                priority: 10,
            },
        ],
    }
}

fn movement_points_template() -> ScaffoldRecipe {
    ScaffoldRecipe {
        template_id: "movement_points".to_string(),
        description: "Movement point properties on tokens and movement cost on cells.".to_string(),
        actions: vec![
            ScaffoldAction::AddProperty {
                entity_name: "Token".to_string(),
                prop_name: "movement_allowance".to_string(),
                prop_type: "int".to_string(),
            },
            ScaffoldAction::AddProperty {
                entity_name: "Token".to_string(),
                prop_name: "movement_remaining".to_string(),
                prop_type: "int".to_string(),
            },
            ScaffoldAction::AddProperty {
                entity_name: "Cell".to_string(),
                prop_name: "movement_cost".to_string(),
                prop_type: "int".to_string(),
            },
            ScaffoldAction::CreateEnum {
                name: "MovementType".to_string(),
                options: vec![
                    "Foot".to_string(),
                    "Mechanized".to_string(),
                    "Motorized".to_string(),
                ],
            },
            ScaffoldAction::AddProperty {
                entity_name: "Token".to_string(),
                prop_name: "movement_type".to_string(),
                prop_type: "enum:MovementType".to_string(),
            },
        ],
    }
}

fn zones_of_control_template() -> ScaffoldRecipe {
    ScaffoldRecipe {
        template_id: "zones_of_control".to_string(),
        description: "ZOC type enum and projection properties for combat units.".to_string(),
        actions: vec![
            ScaffoldAction::CreateEnum {
                name: "ZocType".to_string(),
                options: vec![
                    "None".to_string(),
                    "Rigid".to_string(),
                    "Semi-Rigid".to_string(),
                    "Fluid".to_string(),
                ],
            },
            ScaffoldAction::AddProperty {
                entity_name: "Token".to_string(),
                prop_name: "zoc_type".to_string(),
                prop_type: "enum:ZocType".to_string(),
            },
            ScaffoldAction::AddProperty {
                entity_name: "Token".to_string(),
                prop_name: "zoc_strength".to_string(),
                prop_type: "int".to_string(),
            },
            ScaffoldAction::AddProperty {
                entity_name: "Cell".to_string(),
                prop_name: "zoc_cost".to_string(),
                prop_type: "int".to_string(),
            },
        ],
    }
}

fn stacking_template() -> ScaffoldRecipe {
    ScaffoldRecipe {
        template_id: "stacking".to_string(),
        description: "Stacking limit on cells and stacking value on tokens.".to_string(),
        actions: vec![
            ScaffoldAction::AddProperty {
                entity_name: "Cell".to_string(),
                prop_name: "stacking_limit".to_string(),
                prop_type: "int".to_string(),
            },
            ScaffoldAction::AddProperty {
                entity_name: "Token".to_string(),
                prop_name: "stacking_value".to_string(),
                prop_type: "int".to_string(),
            },
        ],
    }
}

fn terrain_effects_template() -> ScaffoldRecipe {
    ScaffoldRecipe {
        template_id: "terrain_effects".to_string(),
        description: "Terrain type enum and defense/movement modifier properties on cells."
            .to_string(),
        actions: vec![
            ScaffoldAction::CreateEnum {
                name: "TerrainType".to_string(),
                options: vec![
                    "Clear".to_string(),
                    "Forest".to_string(),
                    "Hill".to_string(),
                    "Mountain".to_string(),
                    "Swamp".to_string(),
                    "City".to_string(),
                ],
            },
            ScaffoldAction::AddProperty {
                entity_name: "Cell".to_string(),
                prop_name: "terrain_type".to_string(),
                prop_type: "enum:TerrainType".to_string(),
            },
            ScaffoldAction::AddProperty {
                entity_name: "Cell".to_string(),
                prop_name: "defense_modifier".to_string(),
                prop_type: "int".to_string(),
            },
            ScaffoldAction::AddProperty {
                entity_name: "Cell".to_string(),
                prop_name: "terrain_movement_cost".to_string(),
                prop_type: "int".to_string(),
            },
        ],
    }
}

fn supply_template() -> ScaffoldRecipe {
    ScaffoldRecipe {
        template_id: "supply".to_string(),
        description: "Supply source on cells, supply range and status on tokens.".to_string(),
        actions: vec![
            ScaffoldAction::AddProperty {
                entity_name: "Cell".to_string(),
                prop_name: "supply_source".to_string(),
                prop_type: "bool".to_string(),
            },
            ScaffoldAction::AddProperty {
                entity_name: "Token".to_string(),
                prop_name: "supply_range".to_string(),
                prop_type: "int".to_string(),
            },
            ScaffoldAction::AddProperty {
                entity_name: "Token".to_string(),
                prop_name: "in_supply".to_string(),
                prop_type: "bool".to_string(),
            },
        ],
    }
}
