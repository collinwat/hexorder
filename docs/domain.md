# Hexorder Domain Model

These are the core concepts the product is built around. They emerged from early design
conversations and will be refined as we build.

## Game System (versioned)

The abstract design artifact. Defines how the world works: rules, constraints, unit type
definitions, terrain type definitions, combat mechanics, movement rules, turn phase structure,
theme/aesthetics. This is what gets exported. Multiple games can share one system.

## Game (pinned to a Game System version)

A concrete game built on a specific Game System version. Contains map(s), unit rosters, and playable
configurations. Cannot exist without a Game System.

## Scenario / Campaign / Situation

Different ways to experience a Game. Same rules and content, different setups or progressions. These
"skin" or "configure" the Game to provide distinct play experiences.

## Workspace

The user's persistent design-time context. Remembers which Game System or Game the user was working
on, camera state, open panels, and tool state. The user resumes a workspace when they open Hexorder.

## Game Session

Play-test runtime. The game is running, but the user has extra tooling — note-taking, insight
capture, logging — to feed observations back into the design process.

## Change Isolation Model

- Game Systems are immutable at a given version (v1, v2, v3...).
- Games pin to a specific Game System version.
- A Game can fork/duplicate Game System content to experiment with changes in isolation.
- Integration back into the Game System is deliberate, with impact analysis across all consuming
  Games.
- Each Game opts in to upgrading to a new Game System version.
