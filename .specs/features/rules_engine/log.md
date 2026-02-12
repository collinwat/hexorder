# Feature Log: rules_engine

## 2026-02-11 — Initial spec

- Created feature spec for M4
- Rules engine evaluates constraints against board state
- Computes ValidMoveSet via BFS with constraint evaluation
- Produces SchemaValidation for the game system definition
- Key design: if no constraints exist, all moves are valid (backward compatible with M3)
