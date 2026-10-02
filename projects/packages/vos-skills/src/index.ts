/**
 * User-facing skill catalog for `@game-gpt/vos-skills`.
 * Teaches agents how to help end users author `.vos` application schema.
 */

export type SkillDelivery = "docs-only" | "cli-stub" | "tool-live";

export type VosSkillMeta = {
    readonly id: string;
    readonly name: string;
    readonly description: string;
    readonly skillMd: string;
    readonly delivery: SkillDelivery;
};

export const VOS_SKILLS: readonly VosSkillMeta[] = [
    {
        id: "vos-language",
        name: "vos-language",
        description:
            "Help the user write or review VOS (.vos) schema — tables, classes, services, enums, and references for their application.",
        skillMd: "skills/vos-language/SKILL.md",
        delivery: "docs-only",
    },
] as const;

export function listVosSkills(): readonly VosSkillMeta[] {
    return VOS_SKILLS;
}

export function getVosSkill(id: string): VosSkillMeta | undefined {
    return VOS_SKILLS.find((s) => s.id === id);
}
