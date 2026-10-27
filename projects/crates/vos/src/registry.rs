//! 持久化 identity registry，存 `.vos/identities.json`。
//!
//! 编译器首次遇到一个 Service / Operation / ServiceMethod 时，根据它的
//! canonical path 预先分配一个单调递增、持久化的 `stable_id_hint`，后续
//! 只要 canonical path 不变就复用同一个 hint。rename 时 canonical path
//! 变化 → 新 entry 分配新 hint，旧 hint 继续留在 registry 里但不再被引用
//! （保留 rename 历史，永不删除、永不复用）。

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// `.vos/identities.json` 的 format version。
pub const REGISTRY_FORMAT_VERSION: &str = "vos-identity-registry-v0";
/// 存放 registry 的目录名。
pub const REGISTRY_DIR: &str = ".vos";
/// registry 文件名。
pub const REGISTRY_FILE_NAME: &str = "identities.json";

/// 持久化 identity registry。
///
/// 所有字段公开以便测试，但正常使用应该通过 [`Self::load_or_create`]、
/// [`Self::ensure_hint`] 和 [`Self::save`]。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdentityRegistry {
    /// format version，用于未来迁移或拒绝未知格式。
    pub format_version: String,
    /// canonical path key → stable_id_hint。
    ///
    /// key 格式：
    /// - service   : `svc::<canonical_path joined with ::>`
    /// - method    : `mtd::<service canonical_path>::<method name>`
    /// - operation : `op::<canonical_path joined with ::>`
    pub entries: BTreeMap<String, u64>,
    /// 下一个要分配的 hint（保留 0，从 1 开始）。
    next_hint: u64,
}

impl IdentityRegistry {
    /// 返回 registry 文件在给定 project root 下的路径。
    pub fn registry_path(project_root: &Path) -> std::path::PathBuf {
        project_root.join(REGISTRY_DIR).join(REGISTRY_FILE_NAME)
    }

    /// 加载或创建 registry。
    ///
    /// - 文件存在且 format version 匹配 → 读取并返回
    /// - 文件不存在或 format version 不匹配 → 返回一个空 registry
    pub fn load_or_create(project_root: &Path) -> Result<Self, io::Error> {
        let path = Self::registry_path(project_root);
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Self>(&content) {
                Ok(registry) if registry.format_version == REGISTRY_FORMAT_VERSION => Ok(registry),
                // 版本不匹配或反序列化失败 → 重置为空 registry（不迁移旧格式）
                Ok(_) | Err(_) => Ok(Self::empty()),
            },
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self::empty()),
            Err(err) => Err(err),
        }
    }

    /// 创建一个空 registry（所有 hint 从 1 开始）。
    pub fn empty() -> Self {
        Self {
            format_version: REGISTRY_FORMAT_VERSION.to_owned(),
            entries: BTreeMap::new(),
            next_hint: 1,
        }
    }

    /// 根据 canonical path key 返回已分配的 hint，不存在则分配新的。
    ///
    /// hint 单调递增、永不回退、永不复用。
    pub fn ensure_hint(&mut self, canonical_key: &str) -> u64 {
        if let Some(&hint) = self.entries.get(canonical_key) {
            return hint;
        }
        let hint = self.next_hint;
        self.next_hint = self
            .next_hint
            .checked_add(1)
            .expect("stable_id_hint overflowed u64");
        self.entries.insert(canonical_key.to_owned(), hint);
        hint
    }

    /// 把 registry 写回 `.vos/identities.json`。
    ///
    /// 会自动创建 `.vos/` 目录（如果不存在）。
    pub fn save(&self, project_root: &Path) -> Result<(), io::Error> {
        let dir = project_root.join(REGISTRY_DIR);
        fs::create_dir_all(&dir)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        fs::write(dir.join(REGISTRY_FILE_NAME), content)
    }

    /// 当前已分配的 hint 总数（便于测试断言）。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否没有任何 entry。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 为 service canonical path 构造 registry key。
pub fn service_key(canonical_path: &[String]) -> String {
    format!("svc::{}", canonical_path.join("::"))
}

/// 为 service method 构造 registry key。
pub fn method_key(service_canonical_path: &[String], method_name: &str) -> String {
    format!(
        "mtd::{}::{}",
        service_canonical_path.join("::"),
        method_name
    )
}

/// 为 operation canonical path 构造 registry key。
pub fn operation_key(canonical_path: &[String]) -> String {
    format!("op::{}", canonical_path.join("::"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let mut dir = env::temp_dir();
        dir.push(format!("vos_registry_test_{}_{}", name, std::process::id()));
        // 清理旧的
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn empty_registry_starts_at_one() {
        let mut registry = IdentityRegistry::empty();
        assert!(registry.is_empty());
        assert_eq!(registry.ensure_hint("op::demo::active"), 1);
        assert_eq!(registry.ensure_hint("svc::demo::AuthService"), 2);
        assert_eq!(registry.ensure_hint("op::demo::active"), 1); // 重复 key 不变
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn load_or_create_writes_and_reads_back() {
        let dir = temp_dir("load_create");
        let mut registry = IdentityRegistry::load_or_create(&dir).unwrap();
        assert!(registry.is_empty());

        registry.ensure_hint("svc::demo::IdentityService");
        registry.ensure_hint("mtd::demo::IdentityService::create");
        registry.save(&dir).unwrap();

        // 第二次加载，hint 应该保持
        let loaded = IdentityRegistry::load_or_create(&dir).unwrap();
        assert_eq!(loaded.entries.len(), 2);
        assert_eq!(loaded.entries.get("svc::demo::IdentityService"), Some(&1));
        assert_eq!(
            loaded.entries.get("mtd::demo::IdentityService::create"),
            Some(&2)
        );

        // 删掉测试目录
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rename_gets_new_hint_old_hint_remains() {
        let dir = temp_dir("rename");
        let mut registry = IdentityRegistry::load_or_create(&dir).unwrap();

        // 第一次编译
        registry.ensure_hint("svc::demo::AuthService"); // hint = 1
        registry.ensure_hint("mtd::demo::AuthService::login"); // hint = 2
        registry.save(&dir).unwrap();

        // rename service → 新 canonical path → 新 hint
        registry.ensure_hint("svc::demo::UserService"); // hint = 3 (新增)
        registry.ensure_hint("mtd::demo::UserService::login"); // hint = 4 (新增)
        registry.save(&dir).unwrap();

        let loaded = IdentityRegistry::load_or_create(&dir).unwrap();
        // 旧 entry 仍在 registry 中（不删除）
        assert_eq!(loaded.entries.get("svc::demo::AuthService"), Some(&1));
        assert_eq!(
            loaded.entries.get("mtd::demo::AuthService::login"),
            Some(&2)
        );
        // 新 entry 也在
        assert_eq!(loaded.entries.get("svc::demo::UserService"), Some(&3));
        assert_eq!(
            loaded.entries.get("mtd::demo::UserService::login"),
            Some(&4)
        );
        // 总共有 4 个 entry
        assert_eq!(loaded.len(), 4);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn version_mismatch_resets() {
        let dir = temp_dir("version");
        // 手动写一个版本不匹配的文件
        fs::create_dir_all(dir.join(REGISTRY_DIR)).unwrap();
        fs::write(
            dir.join(REGISTRY_DIR).join(REGISTRY_FILE_NAME),
            r#"{"formatVersion":"old-version","entries":{"op::x":1},"nextHint":2}"#,
        )
        .unwrap();

        let registry = IdentityRegistry::load_or_create(&dir).unwrap();
        // 版本不匹配 → 重置为空
        assert!(registry.is_empty());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn helper_keys() {
        assert_eq!(
            service_key(&["demo".into(), "AuthService".into()]),
            "svc::demo::AuthService"
        );
        assert_eq!(
            method_key(&["demo".into(), "AuthService".into()], "login"),
            "mtd::demo::AuthService::login"
        );
        assert_eq!(
            operation_key(&["demo".into(), "auth".into(), "login".into()]),
            "op::demo::auth::login"
        );
    }
}
