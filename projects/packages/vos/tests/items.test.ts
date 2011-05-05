import { describe, expect, expectTypeOf, it } from "vitest";
import { v } from "../src/builtins.js";
import type { Infer, RefDescriptor } from "../src/descriptor.js";
import type { ReferenceKey, VosFileRef } from "../src/values.js";

const User = v.table("User", {
    user_id: v.uuid().primary(),
    user_name: v.utf8().unique(),
    avatar: v.file().optional(),
});

describe("v.table", () => {
    it("records the declaration name, kind and named fields", () => {
        expect(User.kind).toBe("table");
        expect(User.name).toBe("User");
        expect(Object.keys(User.fields)).toEqual(["user_id", "user_name", "avatar"]);
    });

    it("names every field descriptor", () => {
        expect(User.fields.user_id.name).toBe("user_id");
        expect(User.fields.user_id.kind).toBe("uuid");
        expect(User.fields.user_id.attrs).toEqual([{ name: "primary", args: [] }]);
        expect(User.fields.avatar.name).toBe("avatar");
        expect(User.fields.avatar.kind).toBe("optional");
        expect(User.fields.avatar.inner?.kind).toBe("file");
    });

    it("exposes fields directly as field references", () => {
        expect(User.user_name).toBe(User.fields.user_name);
        expect(User.user_name.attrs).toEqual([{ name: "unique", args: [] }]);
    });

    it("keeps field references out of the enumerable surface", () => {
        expect(Object.keys(User).sort()).toEqual(["fields", "kind", "name"]);
        expect(Object.getOwnPropertyDescriptor(User, "user_id")?.enumerable).toBe(false);
    });

    it("serializes to declarations and fields only", () => {
        expect(JSON.parse(JSON.stringify(User))).toEqual({
            name: "User",
            kind: "table",
            fields: {
                user_id: { name: "user_id", kind: "uuid", attrs: [{ name: "primary", args: [] }] },
                user_name: { name: "user_name", kind: "utf8", attrs: [{ name: "unique", args: [] }] },
                avatar: { name: "avatar", kind: "optional", attrs: [], inner: { kind: "file", attrs: [] } },
            },
        });
    });

    it("keeps a field reference named when attributes are appended", () => {
        const renamed = User.user_name.unique();
        expect(renamed.kind).toBe("utf8");
        expect((renamed as { readonly name?: string }).name).toBe("user_name");
        expect(renamed.attrs).toEqual([
            { name: "unique", args: [] },
            { name: "unique", args: [] },
        ]);
    });
});

describe("v.table row inference", () => {
    it("infers the row shape from the field descriptors", () => {
        expectTypeOf<Infer<typeof User>>().toEqualTypeOf<{
            user_id: string;
            user_name: string;
            avatar: VosFileRef | null;
        }>();
    });

    it("infers a field reference value", () => {
        expectTypeOf<Infer<typeof User.user_id>>().toEqualTypeOf<string>();
    });
});

describe("v.class and v.object", () => {
    it("marks classes as non-persistent named declarations", () => {
        const PublicUser = v.class("PublicUser", { user_id: v.uuid(), display_name: v.utf8() });
        expect(PublicUser.kind).toBe("class");
        expect(PublicUser.name).toBe("PublicUser");
        expectTypeOf<Infer<typeof PublicUser>>().toEqualTypeOf<{ user_id: string; display_name: string }>();
    });

    it("marks objects as anonymous structural shapes", () => {
        const projection = v.object({ user_id: v.uuid(), display_name: v.utf8() });
        expect(projection.kind).toBe("object");
        expect((projection as unknown as Record<string, unknown>).name).toBeUndefined();
        expectTypeOf<Infer<typeof projection>>().toEqualTypeOf<{ user_id: string; display_name: string }>();
    });
});

describe("v.ref", () => {
    it("records the target declaration and infers a primary-key value", () => {
        const author = v.ref(User);
        expect(author.kind).toBe("ref");
        expect(author.target).toEqual({ kind: "table", name: "User" });
        expectTypeOf(author).toEqualTypeOf<RefDescriptor<"User">>();
        expectTypeOf<Infer<typeof author>>().toEqualTypeOf<ReferenceKey>();
    });

    it("accepts a named class as a target", () => {
        const PublicUser = v.class("PublicUser", { user_id: v.uuid() });
        expect(v.ref(PublicUser).target.name).toBe("PublicUser");
    });

    it("refuses an anonymous object target", () => {
        const projection = v.object({ user_id: v.uuid() });
        expect(() => v.ref(projection as unknown as { kind: string; name: string })).toThrow(/named `table` or `class`/);
    });

    it("refuses a reference to something without a declaration name", () => {
        expect(() => v.ref({ kind: "table", name: "Not An Identifier" })).toThrow(/not a valid VOS identifier/);
    });
});

describe("declaration validation", () => {
    it("rejects a non-identifier declaration name", () => {
        expect(() => v.table("User Table", { id: v.uuid() })).toThrow(/not a valid VOS identifier/);
    });

    it("rejects a non-identifier field name", () => {
        expect(() => v.table("User", { "user-id": v.uuid() })).toThrow(/not a valid VOS identifier/);
    });

    it("accepts field names that collide with declaration metadata", () => {
        const Event = v.table("Event", { kind: v.utf8(), fields: v.bool(), name: v.uuid() });

        expect(Event.kind).toBe("table");
        expect(Event.name).toBe("Event");
        expect(Object.keys(Event.fields)).toEqual(["kind", "fields", "name"]);
        expect(Event.fields.kind.name).toBe("kind");
        expect(Event.fields.kind.kind).toBe("utf8");
        expect(Event.fields.name.kind).toBe("uuid");
        expectTypeOf<Infer<typeof Event>>().toEqualTypeOf<{ kind: string; fields: boolean; name: string }>();
    });
});
