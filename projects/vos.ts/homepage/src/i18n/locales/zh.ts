export default {
    brand: {
        short: "VOS",
        full: "Virtual Object Schema",
    },
    nav: {
        home: "首页",
        playground: "Playground",
        try: "试用",
        primary: "主导航",
    },
    locale: {
        label: "语言",
        zh: "中文",
        en: "English",
    },
    home: {
        eyebrow: "语言 · Virtual Object Schema",
        lede: "同一套 schema 语言，覆盖持久化与通信。历史上拆成 Iris（存储）与 Hermes（服务）——语法相同，只是两个面向。",
        openPlayground: "打开 Playground",
        viewPackages: "查看语言面",
        whyTitle: "语言面",
        whyLede: "与 Atlas schema 对齐：Iris + Hermes 是 VOS 的产品面向。",
        features: {
            namespaceTitle: "namespace · using",
            namespaceBody:
                "`namespace a::b` 隔离定义；`using path::Type` 跨文件引入共享领域类型。",
            tableTitle: "table · class",
            tableBody:
                "`table` 是持久化面向；`class` 是共享 DTO / 领域类型。字段按名称寻址，不靠声明顺序。",
            refTitle: "&T 与 T",
            refBody:
                "`&User` 只存主键引用；裸 `User` 内联完整值。可空用 `T?`，列表用 `[T]`。",
            attrTitle: "Attribute",
            attrBody:
                "`[primary]`、`[unique]`、`@@id`、`@slug`，以及 `[post(\"/auth/login\")]`、`[authorize]` 等路由属性。",
            enumsTitle: "enums · flags · union",
            enumsBody:
                "数值枚举、位标志，以及 union / variant 都必须带 `[tag(...)]` 的 tagged union。",
            serviceTitle: "service",
            serviceBody:
                "方法必须命名参数：`login(request: LoginRequest) -> LoginResult`。默认 JSON-RPC；带 route attribute 时生成 REST。`stream<T>` 用于连续事件。",
        },
        packagesTitle: "Packages",
        packagesLede: "TypeScript 在 `@game-gpt`，Rust 在 `projects/vos.rs`。",
        pkgVos: "Schema 工具与轻量源码检查",
        pkgRs: "Rust crates：vos / vos-ast / vos-parser",
        tryPlayground: "去 Playground 试一下",
    },
    playground: {
        eyebrow: "/playground",
        title: "Playground",
        lede: "编辑 VOS schema（持久化或服务面向），高亮预览，并做轻量结构检查。",
        reset: "重置示例",
        source: "source.vos",
        preview: "highlight · everforest-light",
        ok: "结构看起来正常",
        failed: "需要处理",
        hint: "与 Atlas Iris/Hermes 为同一语言。完整解析随 vos-parser 落地；当前检查覆盖花括号、顶层声明与位置参数错误。",
        samplePersistence: "持久化 (table)",
        sampleService: "服务 (service)",
    },
    footer: {
        blurb:
            "Virtual Object Schema —— 用同一套语言描述存储表与通信服务；值文档交给 VON。",
        explore: "探索",
        packages: "包",
        related: "相关",
        von: "VON · Virtual Object Notation",
        copyright: "game-gpt · Virtual Object Schema",
        mono: "table · class · service",
    },
    titles: {
        home: "VOS - Virtual Object Schema",
        playground: "Playground · VOS",
    },
} as const;
