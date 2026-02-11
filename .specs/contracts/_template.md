# Contract: [NAME]

## Purpose

[One sentence: what shared interface does this contract define?]

## Consumers

- [Feature that reads/uses these types]

## Producers

- [Feature that creates/writes these types]

## Types

### Components

```rust
#[derive(Component, Debug, Clone)]
pub struct ExampleComponent {
    pub field: Type,
}
```

### Resources

```rust
#[derive(Resource, Debug)]
pub struct ExampleResource {
    pub field: Type,
}
```

### Events

```rust
#[derive(Event, Debug)]
pub struct ExampleEvent {
    pub field: Type,
}
```

### Enums / Utility Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleEnum {
    VariantA,
    VariantB,
}
```

## Invariants

- [What must always be true about these types?]

## Changelog

| Date | Change             | Reason |
| ---- | ------------------ | ------ |
|      | Initial definition |        |
