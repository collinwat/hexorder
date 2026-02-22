# Plugin Log: Export

## Status: complete

## Decision Log

### 2026-02-21 — printpdf 0.9 API confirmed

**Context**: Evaluating printpdf for counter sheet generation. **Decision**: Use printpdf 0.9 with
built-in Helvetica fonts (BuiltinFont enum). **Rationale**: Built-in fonts eliminate TTF bundling
complexity. Operation-based API (Vec<Op>) is clean for batch PDF generation. 12 transitive
dependency splits added to deny.toml skip list. **Lesson**: Web docs show outdated API; had to read
crate source to find correct `PdfFontHandle::Builtin`, `Op::SetFont` + `Op::ShowText` pattern.

### 2026-02-21 — PDF crate selection

**Context**: The pitch identifies printpdf and genpdf as candidates. Need to choose before
implementing counter sheet generation. **Decision**: Evaluate printpdf first per pitch
recommendation. **Rationale**: printpdf is well-maintained, handles basic layout (rectangles, text,
color fills), and has no unsafe code. Our needs are simple — no complex typography or vector
graphics. **Alternatives rejected**: genpdf (fallback if printpdf insufficient), LaTeX (pitch
explicitly excludes this dependency).

### 2026-02-21 — Research summary

**Context**: Wiki research consumed during kickoff orientation. **Key findings**:

- Community Analysis confirms physical prototyping is central to wargame design workflow
- HexDraw (discontinued Oct 2023) left a critical gap in integrated design-to-physical tools
- LaTeX wargame package is the closest existing integrated solution (requires LaTeX expertise)
- Component Studio demonstrates the target workflow: print-and-play + digital export from one source
- Battle for Moscow is the recommended reference game for complexity validation (39 counters, 1 map)

## Test Results

### 2026-02-21 — Scope 1 (skeleton + trait)

- 7 new tests, all passing (312 total)
- `cargo clippy --all-targets` — zero warnings
- `mise check:boundary` — no boundary violations
- Tests cover: MockExporter trait impl, error Display formatting, data collection from registry,
  empty state handling, trait object safety, flat-top grid orientation

### 2026-02-21 — Scope 2 (counter sheet PDF)

- 6 new tests (13 total export, 318 total)
- `cargo clippy --all-targets` — zero warnings
- `cargo deny check` — all pass (12 new transitive deps added to skip list)
- Tests cover: PDF output validation, all 3 counter sizes, empty state, type-definition fallback,
  property value formatting (numeric, bool), non-displayable type filtering

### 2026-02-22 — Scope 3 (hex map PDF)

- 6 new tests (19 total export, 324 total)
- `cargo clippy --all-targets` — zero warnings
- `mise check:boundary` — no boundary violations
- Tests cover: PDF output validation, empty state rejection, flat-top orientation, all 3 counter
  sizes, board entity coloring, oversized grid rejection

### 2026-02-22 — Scope 4 (editor UI integration)

- 0 new tests (19 total export, 324 total) — scope is UI wiring, not new logic
- `cargo clippy --all-targets` — zero warnings
- `mise check:boundary` — no boundary violations
- Changes: export system now runs both exporters, shows rfd folder picker, writes PDFs, shows toast
  feedback. Export PDF menu item added to File menu. Stale dead_code annotations cleaned up.

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
|         |            |        |          |

## Deferred / Future Work

- Multi-page map tiling (#66 — future scope)
- VASSAL/TTS export (#65)

## Status Updates

| Date       | Status   | Notes                                                                    |
| ---------- | -------- | ------------------------------------------------------------------------ |
| 2026-02-21 | speccing | Initial spec created during kickoff. Research consumed.                  |
| 2026-02-21 | building | Scope 1 complete: skeleton + ExportTarget trait (f61642f). 7 tests pass. |
| 2026-02-21 | building | Scope 2 complete: counter sheet PDF with printpdf (fd9f1e7). 13 tests.   |
| 2026-02-22 | building | Scope 3 complete: hex map PDF with terrain coloring (42c4144). 19 tests. |
| 2026-02-22 | complete | Scope 4 complete: editor UI integration with save dialog (6e33770).      |
