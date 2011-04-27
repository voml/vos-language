import { describe, expect, it } from "vitest";
import { v } from "./builtins.js";
import { emitSource } from "./emit.js";

const User = v.table("User", {
    user_id: v.uuid().primary(),
    user_name: v.utf8().unique(),
    avatar: v.file().optional(),
});

describe("emitSource for a table", () => {
    it("renders canonical DDL with shorthand keys and trailing commas", () => {
        expect(emitSource(User)).toBe(`table User {
    @@user_id: uuid,
    @user_name: utf8,
    avatar: file?,
}
`);
    });

    it("preserves field declaration order", () => {
        const rows = emitSource(User)
            .split("\n")
            .filter((line) => line.startsWith("    "));
        expect(rows.map((line) => line.trim().split(/[:\s]/)[0])).toEqual(["@@user_id", "@user_name", "avatar"]);
    });
});

describe("emitSource type rendering", () => {
    it("renders every builtin with its VOS spelling", () => {
        const All = v.table("All", {
            a: v.i8(),
            b: v.i16(),
            c: v.i32(),
            d: v.i64(),
            e: v.u8(),
            f: v.u16(),
            g: v.u32(),
            h: v.u64(),
            i: v.f32(),
            j: v.f64(),
            k: v.bool(),
            l: v.utf8(),
            m: v.utf16(),
            n: v.uuid(),
            o: v.decimal(),
            p: v.date(),
            q: v.time(),
            r: v.datetimeUtc(),
            s: v.bytes(),
            t: v.file(),
            u: v.vector(768),
        });

        expect(emitSource(All)).toBe(`table All {
    a: i8,
    b: i16,
    c: i32,
    d: i64,
    e: u8,
    f: u16,
    g: u32,
    h: u64,
    i: f32,
    j: f64,
    k: bool,
    l: utf8,
    m: utf16,
    n: uuid,
    o: decimal,
    p: date,
    q: time,
    r: DateTime<UTC>,
    s: bytes,
    t: file,
    u: vector<768>,
}
`);
    });

    it("composes optional, list and reference wrappers", () => {
        const Post = v.table("Post", {
            post_id: v.uuid().primary(),
            author: v.ref(User),
            author_optional: v.ref(User).optional(),
            tags: v.list(v.utf8()),
            nested: v.list(v.list(v.u8())),
        });

        expect(emitSource(Post)).toBe(`table Post {
    @@post_id: uuid,
    author: &User,
    author_optional: &User?,
    tags: [utf8],
    nested: [[u8]],
}
`);
    });

    it("renders non-shorthand attributes as a bracket group", () => {
        const Event = v.table("Event", {
            event_id: v.uuid().primary().attribute("wire"),
            kind: v.utf8().attribute("tag", 1),
            legacy: v.utf8().attribute("backfill", "anonymous").unique(),
        });

        expect(emitSource(Event)).toBe(`table Event {
    [primary, wire] event_id: uuid,
    [tag(1)] kind: utf8,
    [backfill(anonymous), unique] legacy: utf8,
}
`);
    });
});

describe("emitSource for a class and for several declarations", () => {
    it("renders a class declaration", () => {
        const PublicUser = v.class("PublicUser", { user_id: v.uuid(), display_name: v.utf8() });
        expect(emitSource(PublicUser)).toBe(`class PublicUser {
    user_id: uuid,
    display_name: utf8,
}
`);
    });

    it("separates declarations with a blank line", () => {
        const PublicUser = v.class("PublicUser", { user_id: v.uuid() });
        expect(emitSource([User, PublicUser])).toBe(`table User {
    @@user_id: uuid,
    @user_name: utf8,
    avatar: file?,
}

class PublicUser {
    user_id: uuid,
}
`);
    });

    it("requires at least one declaration", () => {
        expect(() => emitSource([])).toThrow(/at least one declaration/);
    });
});

describe("emitSource refuses non-declarations", () => {
    it("refuses an anonymous object", () => {
        const projection = v.object({ user_id: v.uuid() });
        expect(() => emitSource(projection as unknown as Parameters<typeof emitSource>[0])).toThrow(/is not a schema declaration/);
    });
});
