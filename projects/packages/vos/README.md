# 📐 `@voml/vos`

> **VOS (Virtual Object Schema) type modelling for TypeScript** — descriptors for builtin types, wrappers, tables,
> classes and objects, plus the `Infer` projection used to type rows.

Descriptors are author-side data. A host (YYDB, Iris, …) consumes them as a canonical VOS artifact — this package never
renders or parses `.vos` source itself. Source checks live in
[`@voml/vos-language`](https://www.npmjs.com/package/@voml/vos-language).

## 🧱 Model a table

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

## 🔤 Builtins and wrappers

VOS has no wide `string` builtin, so the builder set stays one-to-one with the language:

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

There is deliberately no `v.string()`, no wide `v.number()` and no plain `v.json()`: each of those would hide wire,
layout or validation differences that VOS keeps distinct.

## 🧬 Infer a row type

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

Field symbols and field values stay separate: `User.user_name` is the field descriptor, while `row.user_name` is the
value and only exists inside a host callback.

## 🚧 Scope

- **Modelling only.** No source parser, no CST/AST, no formatter, no CLI. Descriptors must be compiled into a canonical
  VOS artifact before a host consumes them; this package does not render VOS source text.
- **No invented names.** `enums`, `flags`, `union` and the dynamic row builder are added only once their VOS contract
  is frozen.
- **Narrow by design.** For conformance-grade parsing, the resolved contract, catalog identity and code generation,
  use the `vos` crate in this repository's Rust workspace.

---

`@voml/vos-language` — [checks `.vos` source](https://www.npmjs.com/package/@voml/vos-language)
