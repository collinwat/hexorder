---
name: hex-contract
description:
    Create, modify, or consume shared contract types between plugins. Use when a plugin needs to
    expose or consume shared types, when adding cross-plugin communication, or when checking
    contract spec-code parity. Also use when the user invokes /hex-contract.
---

# Contract

Manage shared contract types between plugins — create, change, or consume.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name                 | Value                                        | Description                                    |
| -------------------- | -------------------------------------------- | ---------------------------------------------- |
| `project_root`       | repository root                              | Base directory; all paths are relative to this |
| `contract_guide`     | `{{ project_root }}/docs/guides/contract.md` | Contract protocol, template, conventions       |
| `contracts_spec_dir` | `{{ project_root }}/docs/contracts`          | Contract specification directory               |
| `contracts_src_dir`  | `{{ project_root }}/src/contracts`           | Contract implementation directory              |
| `contracts_mod`      | `{{ contracts_src_dir }}/mod.rs`             | Contract module registry                       |
| `architecture`       | `{{ project_root }}/docs/architecture.md`    | Plugin dependency graph                        |

## 1. Learn the Contract Protocol

Read `{{ contract_guide }}` to extract the contract protocol, template structure, and conventions.
Specifically, find and hold in memory:

- **Spec template** — the required structure for contract spec documents
- **Implementation rules** — what belongs in contract modules (data types only, no systems or logic)
- **Change protocol** — how to propose and coordinate contract changes
- **Spec-code parity requirements** — every type in `{{ contracts_src_dir }}` must have a matching
  spec in `{{ contracts_spec_dir }}`, and vice versa

Do NOT hardcode template structure or protocol steps — always read them fresh from the file.

## Which Workflow?

1. Check `{{ contracts_spec_dir }}` and `{{ contracts_src_dir }}` for existing contracts relevant to
   your work
2. If you need to **consume** an existing contract → read the spec and use the types
3. If you need to **create** a new contract → follow the creation steps below
4. If you need to **change** an existing contract → follow the change steps below

## Creating a New Contract

1. Create `{{ contracts_spec_dir }}/<name>.md` using the template extracted from
   `{{ contract_guide }}`
2. Create a GitHub Issue describing the addition with the `area:contracts` label
3. Implement the types in `{{ contracts_src_dir }}/<name>.rs` — data types only, no systems or logic
4. Register the module in `{{ contracts_mod }}`
5. Run `cargo build` to verify consumers compile
6. Update `{{ architecture }}` dependency graph if needed

## Changing an Existing Contract

1. Create a GitHub Issue describing the change with the `area:contracts` label
2. Update the spec in `{{ contracts_spec_dir }}/<name>.md`
3. Update the implementation in `{{ contracts_src_dir }}/<name>.rs`
4. Run `cargo build` to verify all consumers still compile
5. Check `{{ architecture }}` for affected features
