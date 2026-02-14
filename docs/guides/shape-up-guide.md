# Shape Up — Methodology Reference

This document describes the Shape Up product development methodology and how Hexorder adopts it.
Shape Up was created by Ryan Singer at Basecamp and published in
[Shape Up: Stop Running in Circles and Ship Work that Matters](https://basecamp.com/shapeup).

---

## Overview

Shape Up replaces perpetual backlog-driven sprints with a deliberate cycle-based system built on
three principles:

1. **Fixed time, variable scope.** The deadline is fixed; scope is adjusted to fit. When the cycle
   ends, whatever is complete ships.
2. **Shaping before scheduling.** Work is designed at the right level of abstraction before it
   enters a cycle — concrete enough that teams know what to do, abstract enough that they have room
   to work out details.
3. **No backlogs.** There is no central list of ideas to groom. Important ideas resurface naturally.
   Stale ideas fade away without consuming time.

---

## The Cycle Structure

Shape Up operates in a repeating rhythm:

```
┌──────────────────────────────────┐   ┌──────────────────┐
│         BUILD CYCLE              │   │    COOL-DOWN      │
│         (6 weeks)                │   │    (2 weeks)      │
│                                  │   │                   │
│  Teams build shaped work.        │   │  Recovery.        │
│  No interruptions.               │   │  Bug fixes.       │
│  Ship at the end.                │   │  Shaping.         │
│                                  │   │  Betting table.   │
└──────────────────────────────────┘   └──────────────────┘
                              ↻ repeat
```

- **Build cycle (6 weeks)**: Long enough to build something meaningful, short enough to feel the
  deadline from the start. Teams work uninterrupted on their assigned projects.
- **Cool-down (2 weeks)**: Buffer period between cycles. Teams choose their own work — fixing bugs,
  exploring ideas, prototyping. Shapers prepare pitches. The betting table meets to decide what to
  build next. The end of a cycle is the worst time to plan; cool-down provides the breathing room.

---

## Phase 1: Shaping

Shaping is the behind-the-scenes design work that happens _before_ a project is ready to schedule. A
small senior group defines the solution at the right level of abstraction.

### Who Shapes

Shapers combine three perspectives:

- **Strategic understanding** — what the business needs
- **Design sensibility** — what a good solution looks like
- **Technical literacy** — what is feasible and what is expensive

Shaping is creative, integrative, and private. It happens in parallel to build cycles.

### Properties of Shaped Work

Shaped work has three essential properties:

| Property    | Meaning                                                                         |
| ----------- | ------------------------------------------------------------------------------- |
| **Rough**   | Visibly unfinished. Open spaces remain for the team's judgment and expertise    |
| **Solved**  | The overall solution is worked out at the macro level. Hard thinking is done    |
| **Bounded** | Clear appetite (time budget). Specific exclusions. The team knows where to stop |

### The Right Level of Abstraction

- **Wireframes are too concrete.** They prescribe visual details too early, leaving designers no
  room for creativity and hiding implementation complexity.
- **Words are too abstract.** "Build a calendar view" sounds sensible but nobody knows what it
  entails. Teams cannot make trade-offs.
- **The sweet spot** is between these extremes: rough, solution-level sketches that convey the
  concept without prescribing visual design.

### The Four Steps of Shaping

#### Step 1: Set Boundaries

Before exploring solutions, define how much time and attention the idea deserves.

**Appetite**: The amount of time you are willing to spend, declared upfront. The inverse of an
estimate:

- **Estimates** start with a design and end with a number (_how long will this take?_)
- **Appetites** start with a number and end with a design (_what can we build in this time?_)

Two sizes:

| Size            | Duration  | Team                                  |
| --------------- | --------- | ------------------------------------- |
| **Small Batch** | 1–2 weeks | One designer + one or two programmers |
| **Big Batch**   | 6 weeks   | Same team size                        |

Nothing in between. Projects either fit small batch or big batch.

**Responding to raw ideas**: The default response is _"Interesting. Maybe some day."_ — a soft no
that leaves the door open without commitment. Ideas must be shaped before they earn a place in a
cycle. If the idea matters, it will come back.

**Narrow the problem**: A broad feature request ("we need a calendar") can mean hundreds of things.
Shaping narrows it to a specific problem and the simplest solution.

**Avoid grab-bags**: Bundles of unrelated tasks dressed up as a feature ("redesign the Files
section") have no clear boundaries, no single problem, and no way to know when you are done.

#### Step 2: Find the Elements

The creative work of roughing out a solution. Identify the main components, interactions, and flows
without getting into visual design or implementation details.

**Breadboarding** — captures the _topology_ of a solution (what connects to what) without visual
design:

- **Places**: Screens, dialogs, menus — drawn as a word with an underline
- **Affordances**: Buttons, fields, interface copy — written below the place they belong to
- **Connection lines**: Arrows showing how affordances take the user between places

**Fat marker sketches** — sketches made with such broad strokes that adding fine detail is
physically impossible. Useful when the idea is inherently visual or too complicated for a pure
breadboard. The broadness forces you to stay at the right abstraction level.

**Getting concrete enough**: Play through the use case mentally. Once you can walk through the
user's journey and the elements fit together, move on. You do not need every detail. Leaving details
out preserves room for creativity in later phases.

#### Step 3: Address Risks and Rabbit Holes

Slow down and stress-test the solution for anything that could blow up during implementation.

**Why this matters**: One unanticipated problem that takes two weeks to solve burns a third of the
six-week budget.

**Risk profiles**:

- **Fat-tailed**: Unknowns, unsolved problems, misunderstood interdependencies. Completion time
  could stretch indefinitely.
- **Thin-tailed**: Independent, well-understood parts. Probability distribution is tight. **This is
  the goal of shaping.**

**Process**:

1. Walk through the solution in slow motion, step by step
2. Identify technical unknowns, unsolved design problems, misunderstood interdependencies
3. Present to technical experts — frame it as _just an idea_, not an assignment
4. Mitigate each risk:
    - **Patch the hole**: Specify a particular approach at that tricky spot
    - **Declare out of bounds**: Explicitly exclude certain things
    - **Cut back**: Remove non-essential parts to eliminate the risky area

#### Step 4: Write the Pitch

Package the shaping work into a formal document for the betting table.

**Five ingredients of a pitch**:

| Ingredient       | Purpose                                                                     |
| ---------------- | --------------------------------------------------------------------------- |
| **Problem**      | Why this matters. Frames the work and provides evaluation criteria          |
| **Appetite**     | How much time: Small Batch (1–2 weeks) or Big Batch (6 weeks)               |
| **Solution**     | The shaped solution — breadboards, fat marker sketches, or a combination    |
| **Rabbit Holes** | Specific tricky spots and the decisions made to prevent teams getting stuck |
| **No Gos**       | Functionality deliberately excluded to fit the appetite                     |

Pitches are written asynchronously and reviewed before the betting table meets. If a pitch is not
selected, it is not queued — it can be re-pitched in a future cycle if the problem is still
relevant.

---

## Phase 2: Betting

### Bets, Not Backlogs

Shape Up eliminates the traditional product backlog. Before each cycle, stakeholders look only at
pitches from the last six weeks, or pitches that somebody purposefully revived.

- Everyone can track ideas independently (support keeps its list, product keeps its list)
- None of these lists are direct inputs to the betting process
- If a pitch is not chosen, it is let go — not carried forward
- Important ideas come back naturally, with context, brought by a person

### The Betting Table

A meeting during cool-down where stakeholders review shaped pitches and decide what to commit to
next. Characteristics:

- **Short**: Rarely longer than 1–2 hours
- **Decisive**: No long grooming sessions
- **Small group**: Decision-makers only (e.g., CEO, CTO, senior programmer, product strategist)

**Questions to ask at the betting table**:

1. Does the problem matter?
2. Is the appetite right?
3. Is the solution attractive?
4. Is this the right time?
5. Are the right people available?

### The Circuit Breaker

If a project does not finish in six weeks, it does not automatically get an extension. **By default,
unfinished projects are cancelled.** The circuit breaker prevents runaway projects.

If a project fails to ship, the team re-shapes the problem — looking for a better approach that
avoids whatever rabbit hole they fell into. A new pitch must be brought back to the betting table.

**Two conditions for extending past the deadline** (both must be met):

1. Outstanding tasks are true must-haves that withstood every attempt to cut scope
2. Outstanding work is all downhill — no unsolved problems, pure execution remains

### Team Sizes

- **Big batch**: One project per team for the full six weeks
- **Small batch**: Multiple smaller projects (1–2 weeks each) per team per cycle

### Bug Smash

Once a year, dedicate an entire cycle to fixing bugs. Individual bugs that are too big for cool-down
can be shaped into a pitch and compete at the betting table.

### Product Development Modes

| Mode                | When                                   | Shaping                            | Goal                 |
| ------------------- | -------------------------------------- | ---------------------------------- | -------------------- |
| **R&D Mode**        | Building something brand new           | Fuzzy — learning by building       | Learn, not ship      |
| **Production Mode** | Core decisions made, building features | Standard — shaped pitches          | Shippable each cycle |
| **Cleanup Mode**    | Pre-launch (max 2 cycles)              | Free-for-all — final cut decisions | Ship the release     |

---

## Phase 3: Building

### Hand Over Responsibility

- **Assign projects, not tasks.** The team receives a shaped pitch, not a list of tasks. They are
  responsible for figuring out the approach, defining tasks, and building the solution.
- **Kick-off**: Post the pitch to the team. The team reads it and gets oriented.
- **Getting oriented**: The first few days will not look like "real work." Teams need to acquaint
  themselves with relevant code, think through the pitch, and explore dead ends. This is normal.
- **Done means deployed.** Testing, QA, and deployment happen within the cycle.

### Get One Piece Done

**Integrate vertically, not horizontally.** Instead of building all back-end first, then all
front-end, pick one small piece and build it end-to-end — working UI and working code — in a few
days.

**Three criteria for choosing what to build first**:

1. **Core**: Central to the project concept
2. **Small**: Completable end-to-end in a few days
3. **Novel**: Involves something new, to surface unknowns early

**Imagined tasks vs. discovered tasks**: Tasks conceived before building are "imagined." The real
bulk of work comes from "discovered" tasks that emerge from doing the work.

**Affordances before pixel-perfect**: Programmers need input elements, buttons, and data display
areas — not polished designs. Visual polish comes after the raw affordances are hooked up.

### Map the Scopes

**Scopes** are meaningful parts of the project that can be completed independently in a few days.
They are bigger than tasks but much smaller than the overall project.

- Scopes are **discovered, not planned** — they emerge from the work
- Each scope becomes a to-do list: scope name as the list name, tasks within it
- Scopes integrate front-end and back-end work together
- Finishing a scope means finishing a real slice of the project

**Scope types**:

| Type           | Description                                                                    |
| -------------- | ------------------------------------------------------------------------------ |
| **Layer cake** | Back-end effort is thin and evenly distributed beneath the UI                  |
| **Iceberg**    | One layer dominates (heavy back-end or heavy front-end)                        |
| **Chowder**    | Loose tasks that do not fit any scope (warning sign if > 3–5 items)            |
| **Grab bag**   | Generic names like "front-end" or "bugs" — a smell indicating poor integration |

### Show Progress: The Hill Chart

A visualization shaped like a hill. Each scope is a dot on the hill:

```
          ●  ←── "I don't believe there are unknowns left"
         / \
        /   \
       /     ●  ←── "I know what to do, just need to do it"
      /       \
     /         \
    /           \
   ●             ●  ←── nearly done
   ↑
"I've thought
 about this"

  UPHILL              DOWNHILL
  (figuring out)      (execution)
```

- **Uphill** = uncertainty, unknowns, problem-solving
- **Downhill** = certainty, confidence, clear path forward

**The three thirds of uphill**:

1. "I've thought about this" (approach conceived in your head)
2. "I've validated my approach" (tested against real code/design)
3. "I don't believe there are unknowns left" (top of the hill)

**Key insight**: Solve with hands, not head. Mental theories about solutions often prove more
complicated in practice. Validation requires building.

A dot that does not move signals someone might be stuck. Breaking a stuck scope into smaller scopes
often unblocks it.

### Decide When to Stop

**Scope hammering**: The forceful, deliberate act of cutting scope to fit the time box. Distinguish
must-haves from nice-to-haves continuously, not as a last-minute panic.

**Compare to baseline (compare down)**: Instead of comparing to an imagined ideal, compare to the
current reality. What is the workaround this feature eliminates? The difference between "never good
enough" and "better than what they have now."

**Nice-to-haves**: Tasks to do if time permits, and to cut if it runs out. Usually they never get
built.

### Move On (Cool-Down)

After shipping:

- **Stay cool.** Avoid knee-jerk reactions to feedback. Give it a few days.
- **Clean slate.** Do not commit to changes based on fresh feedback — maintain a clean slate for the
  next cycle by only betting one cycle at a time.
- **Cool-down activities**: Recovery, bug fixes, exploration, shaping, prototyping. Teams choose
  their own work.
- **Ramp-up** (variant): A fixed period before the next cycle where engineering, product, and
  leadership collaborate to explore ideas, run spikes, and fix pressing issues.

---

## Glossary

| Term                    | Definition                                                                           |
| ----------------------- | ------------------------------------------------------------------------------------ |
| **Appetite**            | Time budget declared upfront. The inverse of an estimate                             |
| **Betting table**       | Meeting where stakeholders choose pitches for the next cycle                         |
| **Big batch**           | One project per team for the full six-week cycle                                     |
| **Breadboard**          | Topology sketch showing places, affordances, and connections — no visual design      |
| **Bug smash**           | Annual cycle dedicated entirely to bug fixes                                         |
| **Circuit breaker**     | Automatic cancellation of projects that miss their deadline                          |
| **Cleanup mode**        | Pre-launch free-for-all (max 2 cycles) for finishing, cutting, and polishing         |
| **Compare to baseline** | Measure against current reality, not perfection                                      |
| **Cool-down**           | Two-week buffer between cycles for recovery, bugs, shaping, and betting              |
| **Discovered tasks**    | Tasks that emerge from doing real work (the true bulk of the project)                |
| **Downhill**            | Execution phase on the hill chart — known path, confidence                           |
| **Fat marker sketch**   | Sketch with broad strokes that prevent fine detail — forces right abstraction level  |
| **Getting oriented**    | Initial exploration period at the start of a cycle (~3 days)                         |
| **Grab bag**            | Generic scope name ("bugs", "front-end") indicating poor integration                 |
| **Hill chart**          | Visualization plotting scopes on an uphill/downhill curve to show progress           |
| **Iceberg**             | Scope where one layer (back-end or front-end) dominates                              |
| **Imagined tasks**      | Tasks conceived before building — often unreliable                                   |
| **Kick-off**            | Handoff moment from shaper to building team                                          |
| **Layer cake**          | Scope where back-end is thin and evenly distributed beneath UI                       |
| **Nice-to-have**        | Work that gets cut if time runs out                                                  |
| **No gos**              | Functionality deliberately excluded from a pitch                                     |
| **Pitch**               | Shaped, risk-reduced proposal with Problem, Appetite, Solution, Rabbit Holes, No Gos |
| **Production mode**     | Standard Shape Up flow — shaped pitches, shippable code each cycle                   |
| **R&D mode**            | Fuzzy shaping, learning by building, senior team, goal is learning not shipping      |
| **Rabbit hole**         | Hidden risk that could blow up during implementation                                 |
| **Ramp-up**             | Variant of cool-down focused on collaborative exploration and spike work             |
| **Scope**               | Meaningful, independently completable slice of a project (few days)                  |
| **Scope hammering**     | Forcefully cutting scope to fit the time box                                         |
| **Six-week cycle**      | The fixed time box for all work                                                      |
| **Small batch**         | Multiple smaller projects (1–2 weeks each) per team per cycle                        |
| **Uphill**              | Figuring-it-out phase on the hill chart — uncertainty, unknowns                      |

---

## Sources

- [Shape Up: Stop Running in Circles and Ship Work that Matters](https://basecamp.com/shapeup) (full
  book, free online)
- [Cool-Downs in Shape Up — Practical Guidance](https://jujodi.medium.com/cool-downs-in-shape-up-some-practical-guidance-4f3656ceaaa)
  (Justin Dickow)
- [Farewell Cool-Down, Hello Ramp-Up](https://jujodi.medium.com/farewell-cool-down-hello-ramp-up-da3294b426c9)
  (Justin Dickow)
- [Implement Shape Up: Cheatsheet](https://jujodi.medium.com/implement-shape-up-cheatsheet-8fb08fffcadc)
  (Justin Dickow)
