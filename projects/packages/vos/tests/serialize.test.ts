import { describe, expect, it } from "vitest";
import { v } from "../src/builtins.js";
import { toSource } from "../src/serialize.js";

describe("serialize", () => {
    it("renders a basic table matching the bracket_primary fixture", () => {
        const Account = v.table("Account", {
            id: v.utf8().primary(),
            name: v.utf8().attribute("default", '""'),
        });
        const out = toSource(Account);
        expect(out).toBe(
            [
                "table Account {",
                '    [primary] id: utf8,',
                '    [default("")] name: utf8',
                "}",
                "",
            ].join("\n"),
        );
    });

    it("renders optional + reference + list", () => {
        const User = v.table("User", {
            id: v.uuid().primary(),
            manager: v.ref({ kind: "table", name: "User" }).optional(),
            tags: v.list(v.utf8()),
            login_at: v.datetimeUtc(),
        });
        const out = toSource(User);
        // Smoke-check shape without being strict about whitespace yet.
        expect(out).toContain("table User {");
        expect(out).toContain("[primary] id: uuid");
        expect(out).toContain("manager: &User?");
        expect(out).toContain("tags: [utf8]");
        expect(out).toContain("login_at: datetimeUtc");
    });

    it("handles class declarations and multiple items in one document", () => {
        const LoginRequest = v.class("LoginRequest", {
            user_name: v.utf8(),
            token: v.utf8(),
        });
        const User = v.table("User", {
            id: v.uuid().primary(),
            name: v.utf8().unique(),
        });
        const out = toSource(User, LoginRequest);
        expect(out).toContain("table User {");
        expect(out).toContain("class LoginRequest {");
    });

    it("rejects ref without target", () => {
        // Can't construct malformed refs via the builder — the runtime guards
        // this. The serializer instead panics on missing .inner/.target/.dim
        // so callers see the failure early, close to the authoring site.
        expect(() => {
            toSource({
                kind: "table",
                name: "T" as const,
                fields: {
                    foo: { kind: "list", attrs: [], inner: undefined } as never,
                },
            } as never);
        }).toThrow(/missing/);
    });
});
