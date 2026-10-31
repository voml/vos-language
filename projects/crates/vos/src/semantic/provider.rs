//! Durable identity manifest provider.
//!
//! The provider is the lock-file allocator for schema identities. It keeps a
//! persisted assignment of durable `TypeId` / `FieldId` / virtual slots keyed
//! by canonical paths and names, allocates monotonic identities for entities
//! seen for the first time, and never reuses an identity that was retired.
//! The generated [`vos_types::IdentityManifest`] is the reviewable artifact a
//! local-push project commits; `bind_identity` and `evolve_identity` consume
//! it unchanged.
//!
//! Reconciliation is whole-set: [`IdentityManifestProvider::ensure_units`]
//! retires every entity that is absent from the supplied projections, so a
//! host must pass the complete current entity set, not one file at a time.
//! A rename is never auto-detected; the author declares it explicitly with
//! [`IdentityManifestProvider::rename_type`] or
//! [`IdentityManifestProvider::rename_field`], which preserves the durable
//! identity while the canonical name changes. Externally authored manifests
//! are applied through [`IdentityManifestProvider::apply_manifest`], which
//! fails explicitly on unknown or conflicting identities instead of
//! silently adopting them.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vos_types::{
    ContractDiagnostic, FieldIdentity, IDENTITY_MANIFEST_VERSION, IdentityManifest,
    SchemaProjection, TypeContractKind, TypeIdentity,
};

use crate::registry::REGISTRY_DIR;

/// Format version of the persisted provider state.
pub const IDENTITY_PROVIDER_FORMAT_VERSION: &str = "vos-identity-manifest-provider-v0";
/// Provider state file name inside the `.vos` directory.
pub const IDENTITY_PROVIDER_FILE_NAME: &str = "identity-manifest.json";

/// One durable field assignment held by the provider.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvidedField {
    /// Current canonical field name.
    pub canonical_name: String,
    /// Durable field ID, never reused after retirement.
    pub field_id: u64,
    /// Durable virtual slot, absent when storage never covered the field.
    pub virtual_field_index: Option<u32>,
}

/// One durable type assignment held by the provider.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvidedType {
    /// Current canonical namespace-qualified path.
    pub canonical_path: Vec<String>,
    /// Durable type ID, never reused after retirement.
    pub type_id: u64,
    /// Declaration kind attached to the identity.
    pub kind: TypeContractKind,
    /// Active field assignments keyed by canonical name.
    pub fields: BTreeMap<String, ProvidedField>,
    /// Next monotonic virtual slot for this type.
    next_slot: u32,
}

/// A retired virtual slot that can never be handed out again.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetiredSlot {
    /// Owning durable type identity.
    pub type_id: u64,
    /// Retired virtual slot.
    pub slot: u32,
}

/// Persisted provider state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdentityProviderState {
    /// Provider state format version.
    pub format_version: String,
    /// Active type assignments keyed by canonical path.
    pub types: BTreeMap<String, ProvidedType>,
    /// Next monotonic type ID.
    pub next_type_id: u64,
    /// Next monotonic field ID.
    pub next_field_id: u64,
    /// Retired type IDs that can never be reused.
    pub retired_type_ids: Vec<u64>,
    /// Retired field IDs that can never be reused.
    pub retired_field_ids: Vec<u64>,
    /// Retired virtual slots that can never be reused.
    pub retired_slots: Vec<RetiredSlot>,
}

/// Durable identity manifest provider backed by `.vos/identity-manifest.json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityManifestProvider {
    state: IdentityProviderState,
}

/// Incoming field set: canonical name paired with its declaration span.
type IncomingFields = Vec<(String, core::range::Range<usize>)>;

/// Incoming type set: canonical path, kind and fields.
type IncomingType = (Vec<String>, TypeContractKind, IncomingFields);

fn path_key(path: &[String]) -> String {
    path.join("::")
}

fn provider_diagnostic(
    code: &str,
    message: String,
    span: Option<core::range::Range<usize>>,
) -> ContractDiagnostic {
    ContractDiagnostic {
        code: code.to_owned(),
        message,
        span,
    }
}

impl IdentityProviderState {
    fn empty() -> Self {
        Self {
            format_version: IDENTITY_PROVIDER_FORMAT_VERSION.to_owned(),
            types: BTreeMap::new(),
            next_type_id: 1,
            next_field_id: 1,
            retired_type_ids: Vec::new(),
            retired_field_ids: Vec::new(),
            retired_slots: Vec::new(),
        }
    }

    /// Rejects a persisted state that contradicts its own invariants.
    fn validate(&self) -> Result<(), String> {
        if self.format_version != IDENTITY_PROVIDER_FORMAT_VERSION {
            return Err(format!(
                "unsupported identity provider format version {}",
                self.format_version
            ));
        }
        if self.next_type_id == 0 || self.next_field_id == 0 {
            return Err("identity provider counters must be non-zero".to_owned());
        }
        let retired_types: BTreeSet<u64> = self.retired_type_ids.iter().copied().collect();
        let retired_fields: BTreeSet<u64> = self.retired_field_ids.iter().copied().collect();
        if retired_types.len() != self.retired_type_ids.len()
            || retired_fields.len() != self.retired_field_ids.len()
        {
            return Err("retired identity lists must not repeat IDs".to_owned());
        }
        let mut active_types = BTreeSet::new();
        let mut active_fields = BTreeSet::new();
        for (key, provided) in &self.types {
            if provided.canonical_path.is_empty()
                || provided.canonical_path.iter().any(String::is_empty)
                || path_key(&provided.canonical_path) != *key
                || provided.type_id == 0
                || !active_types.insert(provided.type_id)
                || retired_types.contains(&provided.type_id)
            {
                return Err(format!("invalid active type identity for {key}"));
            }
            let mut slots = BTreeSet::new();
            for (name, field) in &provided.fields {
                if field.canonical_name != *name
                    || field.field_id == 0
                    || !active_fields.insert(field.field_id)
                    || retired_fields.contains(&field.field_id)
                    || field
                        .virtual_field_index
                        .is_some_and(|slot| !slots.insert(slot))
                {
                    return Err(format!("invalid active field identity in {key}"));
                }
            }
        }
        Ok(())
    }
}

impl IdentityManifestProvider {
    /// Creates an empty provider that allocates every identity from one.
    pub fn empty() -> Self {
        Self {
            state: IdentityProviderState::empty(),
        }
    }

    /// Returns the provider state path for a project root.
    pub fn provider_path(project_root: &Path) -> PathBuf {
        PathBuf::from(project_root)
            .join(REGISTRY_DIR)
            .join(IDENTITY_PROVIDER_FILE_NAME)
    }

    /// Loads the persisted provider state.
    ///
    /// A missing file yields an empty provider. A corrupt file or an unknown
    /// format version is an explicit failure: silently resetting durable
    /// identity state would re-allocate IDs that consumers still reference.
    pub fn load_or_create(project_root: &Path) -> Result<Self, io::Error> {
        let path = Self::provider_path(project_root);
        match fs::read_to_string(&path) {
            Ok(content) => {
                let state: IdentityProviderState =
                    serde_json::from_str(&content).map_err(|error| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "corrupt identity provider state at {}: {error}",
                                path.display()
                            ),
                        )
                    })?;
                state.validate().map_err(|reason| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "invalid identity provider state at {}: {reason}",
                            path.display()
                        ),
                    )
                })?;
                Ok(Self { state })
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::empty()),
            Err(error) => Err(error),
        }
    }

    /// Persists the provider state, creating the `.vos` directory when needed.
    pub fn save(&self, project_root: &Path) -> Result<(), io::Error> {
        let dir = PathBuf::from(project_root).join(REGISTRY_DIR);
        fs::create_dir_all(&dir)?;
        let content = serde_json::to_string_pretty(&self.state)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        fs::write(dir.join(IDENTITY_PROVIDER_FILE_NAME), content)
    }

    /// Reconciles one projection and returns the reviewable manifest.
    pub fn ensure(
        &mut self,
        projection: &SchemaProjection,
    ) -> Result<IdentityManifest, Vec<ContractDiagnostic>> {
        self.ensure_units(std::slice::from_ref(&projection))
    }

    /// Reconciles the complete current entity set across source units.
    ///
    /// Entities seen for the first time receive fresh monotonic identities;
    /// entities kept under the same canonical path or name keep their durable
    /// IDs and virtual slots; entities absent from the set are retired and
    /// their identities never reappear. Declaration order inside or across
    /// units is irrelevant: allocation follows a deterministic sort by
    /// canonical path and field name, so reordering source never moves an ID.
    pub fn ensure_units(
        &mut self,
        projections: &[&SchemaProjection],
    ) -> Result<IdentityManifest, Vec<ContractDiagnostic>> {
        let mut diagnostics = Vec::new();
        let mut incoming: BTreeMap<String, IncomingType> = BTreeMap::new();
        for projection in projections {
            for item in &projection.types {
                if item.canonical_path.is_empty()
                    || item.canonical_path.iter().any(String::is_empty)
                {
                    diagnostics.push(provider_diagnostic(
                        "IMP001",
                        "projected type has an empty canonical path".to_owned(),
                        Some(item.span),
                    ));
                    continue;
                }
                let fields = item
                    .fields
                    .iter()
                    .map(|field| (field.canonical_name.clone(), field.span))
                    .collect();
                if incoming
                    .insert(
                        path_key(&item.canonical_path),
                        (item.canonical_path.clone(), item.kind, fields),
                    )
                    .is_some()
                {
                    diagnostics.push(provider_diagnostic(
                        "IMP003",
                        format!("duplicate type path {}", item.canonical_path.join("::")),
                        Some(item.span),
                    ));
                }
            }
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let mut retired_types = BTreeSet::from_iter(self.state.retired_type_ids.iter().copied());
        let mut retired_fields = BTreeSet::from_iter(self.state.retired_field_ids.iter().copied());
        let mut retired_slots = self
            .state
            .retired_slots
            .iter()
            .map(|slot| (slot.type_id, slot.slot))
            .collect::<BTreeSet<_>>();

        for (key, provided) in self.state.types.iter() {
            if !incoming.contains_key(key) {
                retired_types.insert(provided.type_id);
                for field in provided.fields.values() {
                    retired_fields.insert(field.field_id);
                    if let Some(slot) = field.virtual_field_index {
                        retired_slots.insert((provided.type_id, slot));
                    }
                }
            }
        }
        self.state.types.retain(|key, _| incoming.contains_key(key));

        for (key, (canonical_path, kind, fields)) in &incoming {
            let fresh_type = !self.state.types.contains_key(key);
            if fresh_type {
                let type_id = self.state.next_type_id;
                self.state.next_type_id = self
                    .state
                    .next_type_id
                    .checked_add(1)
                    .expect("identity provider type counter exhausted");
                self.state.types.insert(
                    key.clone(),
                    ProvidedType {
                        canonical_path: canonical_path.clone(),
                        type_id,
                        kind: *kind,
                        fields: BTreeMap::new(),
                        next_slot: 0,
                    },
                );
            }
            let provided = self
                .state
                .types
                .get_mut(key)
                .expect("incoming type is present");
            provided.canonical_path = canonical_path.clone();
            provided.kind = *kind;

            let mut field_names = fields
                .iter()
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>();
            field_names.sort();
            let removed = provided
                .fields
                .keys()
                .filter(|name| !field_names.contains(name))
                .cloned()
                .collect::<Vec<_>>();
            for name in removed {
                let field = provided
                    .fields
                    .remove(&name)
                    .expect("key comes from the map");
                retired_fields.insert(field.field_id);
                if let Some(slot) = field.virtual_field_index {
                    retired_slots.insert((provided.type_id, slot));
                }
            }
            for name in &field_names {
                if !provided.fields.contains_key(name) {
                    let field_id = self.state.next_field_id;
                    self.state.next_field_id = self
                        .state
                        .next_field_id
                        .checked_add(1)
                        .expect("identity provider field counter exhausted");
                    let slot = provided.next_slot;
                    provided.next_slot = provided
                        .next_slot
                        .checked_add(1)
                        .expect("identity provider slot counter exhausted");
                    provided.fields.insert(
                        name.clone(),
                        ProvidedField {
                            canonical_name: name.clone(),
                            field_id,
                            virtual_field_index: Some(slot),
                        },
                    );
                }
            }
        }

        self.state.retired_type_ids = retired_types.into_iter().collect();
        self.state.retired_field_ids = retired_fields.into_iter().collect();
        self.state.retired_slots = retired_slots
            .into_iter()
            .map(|(type_id, slot)| RetiredSlot { type_id, slot })
            .collect();

        Ok(self.manifest())
    }

    /// Renders the current active assignments as a reviewable manifest.
    pub fn manifest(&self) -> IdentityManifest {
        let mut types = self
            .state
            .types
            .values()
            .map(|provided| TypeIdentity {
                canonical_path: provided.canonical_path.clone(),
                type_id: provided.type_id,
                kind: provided.kind,
                fields: {
                    let mut fields = provided
                        .fields
                        .values()
                        .map(|field| FieldIdentity {
                            canonical_name: field.canonical_name.clone(),
                            field_id: field.field_id,
                            virtual_field_index: field.virtual_field_index,
                        })
                        .collect::<Vec<_>>();
                    fields.sort_by_key(|field| field.field_id);
                    fields
                },
            })
            .collect::<Vec<_>>();
        types.sort_by_key(|item| item.type_id);
        IdentityManifest {
            format_version: IDENTITY_MANIFEST_VERSION.to_owned(),
            types,
        }
    }

    /// Renames a type while preserving its durable identity and field IDs.
    ///
    /// The rename must be declared by the author: the provider never infers
    /// that a disappeared path and a fresh path are the same entity.
    pub fn rename_type(
        &mut self,
        from: &[String],
        to: &[String],
    ) -> Result<(), Vec<ContractDiagnostic>> {
        let mut diagnostics = Vec::new();
        let from_key = path_key(from);
        let to_key = path_key(to);
        if to.is_empty() || to.iter().any(String::is_empty) {
            diagnostics.push(provider_diagnostic(
                "IMP001",
                "rename target must have non-empty canonical segments".to_owned(),
                None,
            ));
        }
        if !self.state.types.contains_key(&from_key) {
            diagnostics.push(provider_diagnostic(
                "IMP002",
                format!("unknown type path {from_key}"),
                None,
            ));
        }
        if self.state.types.contains_key(&to_key) {
            diagnostics.push(provider_diagnostic(
                "IMP003",
                format!("rename target {to_key} already has a durable identity"),
                None,
            ));
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        let mut provided = self
            .state
            .types
            .remove(&from_key)
            .expect("validated known source path");
        provided.canonical_path = to.to_vec();
        self.state.types.insert(to_key, provided);
        Ok(())
    }

    /// Renames a field while preserving its durable field ID and slot.
    pub fn rename_field(
        &mut self,
        type_path: &[String],
        from: &str,
        to: &str,
    ) -> Result<(), Vec<ContractDiagnostic>> {
        let mut diagnostics = Vec::new();
        let type_key = path_key(type_path);
        if from.trim().is_empty() || to.trim().is_empty() {
            diagnostics.push(provider_diagnostic(
                "IMP001",
                "rename requires non-empty field names".to_owned(),
                None,
            ));
        }
        let provided = match self.state.types.get(&type_key) {
            Some(provided) => provided,
            None => {
                diagnostics.push(provider_diagnostic(
                    "IMP002",
                    format!("unknown type path {type_key}"),
                    None,
                ));
                return Err(diagnostics);
            }
        };
        if !provided.fields.contains_key(from) {
            diagnostics.push(provider_diagnostic(
                "IMP002",
                format!("unknown field {type_key}::{from}"),
                None,
            ));
        }
        if provided.fields.contains_key(to) {
            diagnostics.push(provider_diagnostic(
                "IMP003",
                format!("field {type_key}::{to} already has a durable identity"),
                None,
            ));
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        let provided = self
            .state
            .types
            .get_mut(&type_key)
            .expect("validated known type path");
        let mut field = provided
            .fields
            .remove(from)
            .expect("validated known field name");
        field.canonical_name = to.to_owned();
        provided.fields.insert(to.to_owned(), field);
        Ok(())
    }

    /// Adopts an externally authored manifest into the provider state.
    ///
    /// Assignments that agree with the provider are kept. An ID that collides
    /// with a different active identity, with a retired identity or with a
    /// retired slot fails explicitly; nothing is silently re-allocated or
    /// overwritten.
    pub fn apply_manifest(
        &mut self,
        manifest: &IdentityManifest,
    ) -> Result<(), Vec<ContractDiagnostic>> {
        let mut diagnostics = Vec::new();
        if manifest.format_version != IDENTITY_MANIFEST_VERSION {
            diagnostics.push(provider_diagnostic(
                "IMP004",
                format!(
                    "unsupported identity manifest version {}",
                    manifest.format_version
                ),
                None,
            ));
            return Err(diagnostics);
        }
        let retired_types = BTreeSet::from_iter(self.state.retired_type_ids.iter().copied());
        let retired_fields = BTreeSet::from_iter(self.state.retired_field_ids.iter().copied());
        let retired_slots = self
            .state
            .retired_slots
            .iter()
            .map(|slot| (slot.type_id, slot.slot))
            .collect::<BTreeSet<_>>();
        let mut active_type_ids = BTreeSet::new();
        let mut active_field_ids = BTreeSet::new();
        for provided in self.state.types.values() {
            active_type_ids.insert(provided.type_id);
            for field in provided.fields.values() {
                active_field_ids.insert(field.field_id);
            }
        }

        let mut incoming_paths = BTreeSet::new();
        for identity in &manifest.types {
            let key = path_key(&identity.canonical_path);
            if identity.canonical_path.is_empty()
                || identity.canonical_path.iter().any(String::is_empty)
                || !incoming_paths.insert(key.clone())
            {
                diagnostics.push(provider_diagnostic(
                    "IMP001",
                    format!("empty or duplicate type path {key} in manifest"),
                    None,
                ));
                continue;
            }
            if identity.type_id == 0 {
                diagnostics.push(provider_diagnostic(
                    "IMP011",
                    format!("zero type ID for {key}"),
                    None,
                ));
            }
            match self.state.types.get(&key) {
                Some(provided) => {
                    if provided.type_id != identity.type_id {
                        diagnostics.push(provider_diagnostic(
                            "IMP005",
                            format!(
                                "manifest assigns type {} to {key} but the provider holds {}",
                                identity.type_id, provided.type_id
                            ),
                            None,
                        ));
                    }
                }
                None => {
                    if retired_types.contains(&identity.type_id) {
                        diagnostics.push(provider_diagnostic(
                            "IMP008",
                            format!(
                                "type ID {} was retired and cannot be adopted",
                                identity.type_id
                            ),
                            None,
                        ));
                    } else if active_type_ids.contains(&identity.type_id) {
                        diagnostics.push(provider_diagnostic(
                            "IMP008",
                            format!(
                                "type ID {} already belongs to another path",
                                identity.type_id
                            ),
                            None,
                        ));
                    }
                }
            }

            let mut names = BTreeSet::new();
            let mut slots = BTreeSet::new();
            for field in &identity.fields {
                if field.canonical_name.trim().is_empty()
                    || !names.insert(field.canonical_name.clone())
                {
                    diagnostics.push(provider_diagnostic(
                        "IMP001",
                        format!("empty or duplicate field name in {key}"),
                        None,
                    ));
                    continue;
                }
                if field.field_id == 0 {
                    diagnostics.push(provider_diagnostic(
                        "IMP011",
                        format!("zero field ID {} in {key}", field.field_id),
                        None,
                    ));
                }
                if let Some(existing) = self
                    .state
                    .types
                    .get(&key)
                    .and_then(|provided| provided.fields.get(&field.canonical_name))
                {
                    if existing.field_id != field.field_id {
                        diagnostics.push(provider_diagnostic(
                            "IMP006",
                            format!(
                                "manifest assigns field {} to {key}::{} but the provider holds {}",
                                field.field_id, field.canonical_name, existing.field_id
                            ),
                            None,
                        ));
                    }
                    if existing.virtual_field_index != field.virtual_field_index {
                        diagnostics.push(provider_diagnostic(
                            "IMP007",
                            format!(
                                "manifest assigns slot {:?} to {key}::{} but the provider holds {:?}",
                                field.virtual_field_index, field.canonical_name, existing.virtual_field_index
                            ),
                            None,
                        ));
                    }
                } else {
                    if retired_fields.contains(&field.field_id)
                        || active_field_ids.contains(&field.field_id)
                    {
                        diagnostics.push(provider_diagnostic(
                            "IMP009",
                            format!(
                                "field ID {} is already active or retired and cannot be adopted",
                                field.field_id
                            ),
                            None,
                        ));
                    }
                    if let Some(slot) = field.virtual_field_index
                        && (retired_slots.contains(&(identity.type_id, slot))
                            || !slots.insert(slot))
                    {
                        diagnostics.push(provider_diagnostic(
                            "IMP010",
                            format!(
                                "slot {slot} is retired or duplicated for type {}",
                                identity.type_id
                            ),
                            None,
                        ));
                    }
                }
            }
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let manifest_paths = manifest
            .types
            .iter()
            .map(|identity| path_key(&identity.canonical_path))
            .collect::<BTreeSet<_>>();
        let mut retired_types = BTreeSet::from_iter(self.state.retired_type_ids.iter().copied());
        let mut retired_fields = BTreeSet::from_iter(self.state.retired_field_ids.iter().copied());
        let mut retired_slots = self
            .state
            .retired_slots
            .iter()
            .map(|slot| (slot.type_id, slot.slot))
            .collect::<BTreeSet<_>>();
        for (key, provided) in self.state.types.iter() {
            if !manifest_paths.contains(key) {
                retired_types.insert(provided.type_id);
                for field in provided.fields.values() {
                    retired_fields.insert(field.field_id);
                    if let Some(slot) = field.virtual_field_index {
                        retired_slots.insert((provided.type_id, slot));
                    }
                }
            }
        }
        self.state
            .types
            .retain(|key, _| manifest_paths.contains(key));

        for identity in &manifest.types {
            let key = path_key(&identity.canonical_path);
            let provided = self.state.types.entry(key).or_insert_with(|| ProvidedType {
                canonical_path: identity.canonical_path.clone(),
                type_id: identity.type_id,
                kind: identity.kind,
                fields: BTreeMap::new(),
                next_slot: 0,
            });
            provided.kind = identity.kind;
            provided.canonical_path = identity.canonical_path.clone();
            let manifest_names = identity
                .fields
                .iter()
                .map(|field| field.canonical_name.clone())
                .collect::<BTreeSet<_>>();
            let removed = provided
                .fields
                .keys()
                .filter(|name| !manifest_names.contains(*name))
                .cloned()
                .collect::<Vec<_>>();
            for name in removed {
                let field = provided
                    .fields
                    .remove(&name)
                    .expect("key comes from the map");
                retired_fields.insert(field.field_id);
                if let Some(slot) = field.virtual_field_index {
                    retired_slots.insert((provided.type_id, slot));
                }
            }
            provided.fields.clear();
            let mut max_slot = 0u32;
            for field in &identity.fields {
                max_slot = max_slot.max(field.virtual_field_index.map_or(0, |slot| slot + 1));
                provided.fields.insert(
                    field.canonical_name.clone(),
                    ProvidedField {
                        canonical_name: field.canonical_name.clone(),
                        field_id: field.field_id,
                        virtual_field_index: field.virtual_field_index,
                    },
                );
            }
            provided.next_slot = provided.next_slot.max(max_slot);
            self.state.next_type_id = self.state.next_type_id.max(identity.type_id + 1);
            for field in &identity.fields {
                self.state.next_field_id = self.state.next_field_id.max(field.field_id + 1);
            }
        }
        self.state.retired_type_ids = retired_types.into_iter().collect();
        self.state.retired_field_ids = retired_fields.into_iter().collect();
        self.state.retired_slots = retired_slots
            .into_iter()
            .map(|(type_id, slot)| RetiredSlot { type_id, slot })
            .collect();
        Ok(())
    }

    /// Returns the durable type ID currently held for a canonical path.
    pub fn type_id(&self, canonical_path: &[String]) -> Option<u64> {
        self.state
            .types
            .get(&path_key(canonical_path))
            .map(|provided| provided.type_id)
    }

    /// Returns the durable field ID currently held for a type path and name.
    pub fn field_id(&self, canonical_path: &[String], field_name: &str) -> Option<u64> {
        self.state
            .types
            .get(&path_key(canonical_path))
            .and_then(|provided| provided.fields.get(field_name))
            .map(|field| field.field_id)
    }

    /// Retired type IDs that will never be allocated again.
    pub fn retired_type_ids(&self) -> &[u64] {
        &self.state.retired_type_ids
    }

    /// Retired field IDs that will never be allocated again.
    pub fn retired_field_ids(&self) -> &[u64] {
        &self.state.retired_field_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_roundtrip(state: &IdentityProviderState) {
        let json = serde_json::to_string(state).unwrap();
        assert_eq!(
            serde_json::from_str::<IdentityProviderState>(&json).unwrap(),
            *state
        );
    }

    #[test]
    fn empty_state_roundtrips_and_validates() {
        let state = IdentityProviderState::empty();
        state_roundtrip(&state);
        state.validate().unwrap();
    }

    #[test]
    fn unknown_format_version_is_an_explicit_load_failure() {
        let dir = std::env::temp_dir().join(format!("vos-provider-version-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(REGISTRY_DIR)).unwrap();
        fs::write(
            dir.join(REGISTRY_DIR).join(IDENTITY_PROVIDER_FILE_NAME),
            r#"{"formatVersion":"bogus","types":{},"nextTypeId":1,"nextFieldId":1,"retiredTypeIds":[],"retiredFieldIds":[],"retiredSlots":[]}"#,
        )
        .unwrap();
        let error = IdentityManifestProvider::load_or_create(&dir).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn corrupt_state_is_an_explicit_load_failure() {
        let dir = std::env::temp_dir().join(format!("vos-provider-corrupt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(REGISTRY_DIR)).unwrap();
        fs::write(
            dir.join(REGISTRY_DIR).join(IDENTITY_PROVIDER_FILE_NAME),
            "not json",
        )
        .unwrap();
        let error = IdentityManifestProvider::load_or_create(&dir).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        fs::remove_dir_all(&dir).unwrap();
    }
}
