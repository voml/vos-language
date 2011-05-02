# `@voml/vos`

VOS declaration and type modelling for TypeScript: descriptors for VOS builtin types, wrappers, tables, classes and
objects, plus the `Infer` projection used to type rows.

Source checking and language diagnostics are **not** part of this package — they live in
[`@voml/vos-language`](../vos-language).

Try the interactive playground on the [VOS homepage](https://vos-language.pages.dev/playground).

## Install

```bash
pnpm add @voml/vos
```

## Model a table

```ts
import { v } from "@voml/vos";

const User = v.table("User", {
    user_id: v.uuid().primary(),
    user_name: v.utf8().unique(),
    avatar: v.file().optional(),
});

const PublicUser = v.class("PublicUser", {
    user_id: v.uuid(),
    name: v.utf8(),
});
```

`table` carries persistence identity, `class` is a non-persistent DTO, and `object` is a structural object used for
nested values, projection results and parameters. One `object` never stands in for all three.

## Builtins and wrappers

VOS has no wide `string` builtin, so the builder set stays one-to-one with VOS:

```ts
v.i8() v.i16() v.i32() v.i64()
v.u8() v.u16() v.u32() v.u64()
v.f32() v.f64()
v.bool()
v.utf8() v.utf16()
v.uuid()
v.decimal()
v.date() v.time() v.datetimeUtc()
v.bytes()
v.vector(3)
v.file()

v.optional(v.utf8())        // T?
v.list(v.uuid())            // [T]
v.ref(User)                 // &T

v.utf8().optional()         // chained equivalent, for field literals
v.uuid().primary()          // @@id
v.utf8().unique()           // @name
v.utf8().attribute("wire")  // any [attr] / [attr(args)]
```

There is deliberately no `v.string()`, no wide `v.number()` and no plain `v.json()`: those would hide the wire,
layout and validation differences that VOS keeps distinct.

## Infer a row type

Descriptors carry runtime identity, so the row type is derived from the descriptor instead of being restated:

```ts
import { v, type Infer } from "@voml/vos";

const User = v.table("User", {
    user_id: v.uuid().primary(),
    user_name: v.utf8(),
});

type UserRow = Infer<typeof User>;

const row: UserRow = { user_id: "u_1", user_name: "Ada" };
```

Field symbols and field values are separate: `User.user_name` is the field descriptor, while `row.user_name` is the
value and only exists inside a host callback.

## Emit canonical VOS source

Descriptors are author-side data. `emitSource` renders a declaration descriptor as canonical VOS source, which the
VOS pipeline then parses and checks:

```ts
import { emitSource, v } from "@voml/vos";

const User = v.table("User", { user_id: v.uuid().primary() });
const source = emitSource(User);
```

An anonymous `object` is not a schema declaration and is rejected by `emitSource`; declare it with `v.class` instead.

## Scope

This package is the modelling surface. It never parses VOS source, never owns a CST/AST, never formats, never manages
database lifecycle, and provides no CLI. Descriptors must be compiled into canonical VOS source or a canonical VOS
artifact before a host consumes them. More declaration items (`enums`, `flags`, `union`, and the dynamic row builder)
are added only once their VOS contract is frozen — this package does not invent names ahead of the language.

Use the Rust [`vos` facade](../../crates/vos) for conformance-grade parsing. Source checks for TypeScript hosts live in
[`@voml/vos-language`](../vos-language).

## Development

```bash
pnpm build
pnpm typecheck
pnpm test
```

See the [TypeScript workspace guide](../README.md) and main [VOS overview](../../../readme.md) for the surrounding
project.
