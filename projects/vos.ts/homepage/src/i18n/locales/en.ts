export default {
    brand: {
        short: "VOS",
        full: "Virtual Object Schema",
    },
    nav: {
        home: "Home",
        playground: "Playground",
        try: "Try it",
        primary: "Primary",
    },
    locale: {
        label: "Language",
        zh: "中文",
        en: "English",
    },
    home: {
        eyebrow: "Language · Virtual Object Schema",
        lede: "One schema language for persistence and communication. Historically split as Iris (storage) and Hermes (services) — same grammar, two facets.",
        openPlayground: "Open Playground",
        viewPackages: "View surface",
        whyTitle: "Language surface",
        whyLede: "Aligned with Atlas schema: Iris + Hermes are product facets of VOS.",
        features: {
            namespaceTitle: "namespace · using",
            namespaceBody:
                "`namespace a::b` isolates definitions. `using path::Type` imports shared domain types across files.",
            tableTitle: "table · class",
            tableBody:
                "`table` is the persistence facet; `class` is the shared DTO / domain type. Fields are name-addressed, not ordinal.",
            refTitle: "&T vs T",
            refBody:
                "`&User` stores a primary-key reference; bare `User` inlines the value. Optional with `T?`, lists with `[T]`.",
            attrTitle: "Attributes",
            attrBody:
                "`[primary]`, `[unique]`, `@@id`, `@slug`, route attrs like `[post(\"/auth/login\")]`, and `[authorize]`.",
            enumsTitle: "enums · flags · union",
            enumsBody:
                "Numeric enums, bitflags, and tagged unions with `[tag(...)]` on the union and every variant.",
            serviceTitle: "service",
            serviceBody:
                "Named parameters only: `login(request: LoginRequest) -> LoginResult`. Default JSON-RPC; REST when a route attribute is present. `stream<T>` for continuous events.",
        },
        packagesTitle: "Packages",
        packagesLede: "TypeScript workspace under `@game-gpt`, Rust under `projects/vos.rs`.",
        pkgVos: "Schema toolkit and lightweight source checks",
        pkgRs: "Rust crates: vos / vos-ast / vos-parser",
        tryPlayground: "Try in Playground",
    },
    playground: {
        eyebrow: "/playground",
        title: "Playground",
        lede: "Edit a VOS schema (persistence or service facet), highlight it, and run a lightweight structural check.",
        reset: "Reset sample",
        source: "source.vos",
        preview: "highlight · everforest-light",
        ok: "Looks structural",
        failed: "Needs attention",
        hint: "Same language as Atlas Iris/Hermes. Full parse lands with vos-parser; this check covers braces, top-level items, and positional service parameters.",
        samplePersistence: "Persistence (table)",
        sampleService: "Service",
    },
    footer: {
        blurb:
            "Virtual Object Schema — one language for storage tables and communication services. Pairs with VON for values.",
        explore: "Explore",
        packages: "Packages",
        related: "Related",
        von: "VON · Virtual Object Notation",
        copyright: "game-gpt · Virtual Object Schema",
        mono: "table · class · service",
    },
    titles: {
        home: "VOS - Virtual Object Schema",
        playground: "Playground · VOS",
    },
} as const;
