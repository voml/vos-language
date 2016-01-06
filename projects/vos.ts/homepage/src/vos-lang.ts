import type {LanguageRegistration} from "shiki";

/** TextMate grammar aligned with Atlas schema (Iris + Hermes facets). */
export const vosLanguage: LanguageRegistration = {
    name: "vos",
    scopeName: "source.vos",
    patterns: [
        {include: "#comment"},
        {include: "#attribute"},
        {include: "#string"},
        {include: "#number"},
        {include: "#keyword"},
        {include: "#builtin"},
        {include: "#reference"},
        {include: "#field"},
        {include: "#punctuation"},
    ],
    repository: {
        comment: {
            name: "comment.line.number-sign.vos",
            match: "#.*$",
        },
        attribute: {
            name: "meta.attribute.vos",
            begin: "\\[",
            end: "\\]",
            patterns: [
                {
                    name: "entity.name.function.attribute.vos",
                    match:
                        "\\b(primary|unique|unknown|tag|authorize|get|post|put|patch|delete|obsolete)\\b",
                },
                {
                    name: "string.quoted.double.vos",
                    begin: '"',
                    end: '"',
                },
                {
                    name: "constant.numeric.vos",
                    match: "\\b\\d+\\b",
                },
            ],
        },
        string: {
            name: "string.quoted.double.vos",
            begin: '"',
            end: '"',
            patterns: [{name: "constant.character.escape.vos", match: '\\\\.'}],
        },
        number: {
            name: "constant.numeric.vos",
            match: "\\b0x[0-9A-Fa-f]+\\b|\\b\\d+\\b",
        },
        keyword: {
            name: "keyword.control.vos",
            match:
                "\\b(namespace|using|table|class|enums|flags|union|service|const|obsolete|stream)\\b",
        },
        builtin: {
            name: "support.type.vos",
            match:
                "\\b(utf8|bool|i64|f64|uuid|decimal|d128|DateTime|UTC|null|true|false)\\b",
        },
        reference: {
            name: "storage.modifier.reference.vos",
            match: "&",
        },
        field: {
            name: "variable.other.property.vos",
            match: "@@?[A-Za-z_][A-Za-z0-9_]*(?=\\s*:)",
        },
        punctuation: {
            patterns: [
                {name: "punctuation.separator.key-value.vos", match: ":"},
                {name: "keyword.operator.arrow.vos", match: "->"},
                {name: "keyword.operator.optional.vos", match: "\\?"},
                {name: "punctuation.definition.block.begin.vos", match: "\\{"},
                {name: "punctuation.definition.block.end.vos", match: "\\}"},
                {name: "punctuation.definition.list.begin.vos", match: "\\["},
                {name: "punctuation.definition.list.end.vos", match: "\\]"},
                {name: "punctuation.separator.namespace.vos", match: "::"},
                {name: "punctuation.separator.comma.vos", match: ","},
                {name: "punctuation.terminator.statement.vos", match: ";"},
                {name: "punctuation.definition.parameters.begin.vos", match: "\\("},
                {name: "punctuation.definition.parameters.end.vos", match: "\\)"},
                {name: "keyword.operator.assignment.vos", match: "="},
            ],
        },
    },
};
