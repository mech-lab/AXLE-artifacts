use axle_artifact::ArtifactDirectoryExt;
use axle_core::{
    AxleArtifact, DeclarationKind, Diagnostic, DiagnosticLevel, Environment, VerificationMode,
    VerificationResultStatus, VerificationStatus,
};
use axle_hash::{Digest, HashError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const GRAPH_SCHEMA_V0: &str = "axle.graph.v0";
pub const DIFF_SCHEMA_V0: &str = "axle.diff.v0";

const EDGE_SOURCE: &str = "source";
const EDGE_ENVIRONMENT: &str = "environment";
const EDGE_VERIFICATION: &str = "verification";
const EDGE_DECLARATION: &str = "declaration";
const EDGE_DIAGNOSTIC: &str = "diagnostic";
const EDGE_STATEMENT: &str = "statement";
const EDGE_BODY: &str = "body";
const EDGE_DEPENDENCY: &str = "dependency";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleGraph {
    pub schema: String,
    pub root: Digest,
    pub nodes: Vec<MerkleNode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleNode {
    pub id: Digest,
    pub kind: NodeKind,
    pub label: String,
    pub payload: NodePayload,
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub label: String,
    pub target: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Artifact,
    Source,
    Environment,
    Verification,
    Declaration,
    Statement,
    Body,
    Diagnostic,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodePayload {
    Artifact {
        artifact_id: Digest,
    },
    Source {
        language: String,
        module: Option<String>,
        path: Option<String>,
        source_digest: Digest,
    },
    Environment {
        engine: String,
        engine_version: Option<String>,
        lean_version: Option<String>,
        mathlib_digest: Option<Digest>,
        environment_digest: Digest,
    },
    Verification {
        mode: VerificationMode,
        status: VerificationResultStatus,
        formal_statement_digest: Digest,
        failed_declarations: Vec<String>,
        verification_digest: Digest,
    },
    Declaration {
        name: String,
        declaration_kind: DeclarationKind,
        verification_status: VerificationStatus,
    },
    Statement {
        digest: Option<Digest>,
    },
    Body {
        digest: Option<Digest>,
    },
    Diagnostic {
        level: DiagnosticLevel,
        message: String,
        code: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDiff {
    pub schema: String,
    pub identical: bool,
    pub old_root: Digest,
    pub new_root: Digest,
    pub old_source: Digest,
    pub new_source: Digest,
    pub source_changed: bool,
    pub old_environment: Digest,
    pub new_environment: Digest,
    pub environment_changed: bool,
    pub old_verification: Option<Digest>,
    pub new_verification: Option<Digest>,
    pub verification_changed: bool,
    #[serde(default)]
    pub added_declarations: Vec<String>,
    #[serde(default)]
    pub removed_declarations: Vec<String>,
    #[serde(default)]
    pub changed_declarations: Vec<ChangedDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangedDeclaration {
    pub name: String,
    pub old_node_id: Digest,
    pub new_node_id: Digest,
    pub statement_changed: bool,
    pub body_changed: bool,
    pub verification_status_changed: bool,
    pub dependencies_changed: bool,
}

#[derive(Clone, Debug)]
struct DeclarationDraft {
    name: String,
    kind: DeclarationKind,
    verification_status: VerificationStatus,
    statement: Option<Digest>,
    body: Option<Digest>,
    dependencies: Vec<String>,
}

#[derive(Clone, Debug)]
struct DeclarationView {
    node_id: Digest,
    name: String,
    verification_status: VerificationStatus,
    statement_target: Option<Digest>,
    body_target: Option<Digest>,
    dependency_targets: Vec<Digest>,
}

impl MerkleGraph {
    pub fn derive(artifact: &AxleArtifact) -> Result<Self, GraphError> {
        let root = artifact.artifact_digest()?;
        let source_id = Digest::from_canonical_json(&artifact.source)?;
        let environment_id = digest_environment(&artifact.manifest.environment)?;
        let verification_id = artifact
            .verification
            .as_ref()
            .map(Digest::from_canonical_json)
            .transpose()?;

        let mut nodes_by_id = BTreeMap::new();

        insert_node(
            &mut nodes_by_id,
            MerkleNode {
                id: source_id.clone(),
                kind: NodeKind::Source,
                label: "source".to_owned(),
                payload: NodePayload::Source {
                    language: artifact.source.language.clone(),
                    module: artifact.source.module.clone(),
                    path: artifact.source.path.clone(),
                    source_digest: source_id.clone(),
                },
                edges: Vec::new(),
            },
        )?;

        insert_node(
            &mut nodes_by_id,
            MerkleNode {
                id: environment_id.clone(),
                kind: NodeKind::Environment,
                label: "environment".to_owned(),
                payload: NodePayload::Environment {
                    engine: artifact.manifest.environment.engine.clone(),
                    engine_version: artifact.manifest.environment.engine_version.clone(),
                    lean_version: artifact.manifest.environment.lean_version.clone(),
                    mathlib_digest: artifact.manifest.environment.mathlib_digest.clone(),
                    environment_digest: environment_id.clone(),
                },
                edges: Vec::new(),
            },
        )?;

        if let (Some(verification), Some(verification_id)) =
            (artifact.verification.as_ref(), verification_id.as_ref())
        {
            insert_node(
                &mut nodes_by_id,
                MerkleNode {
                    id: verification_id.clone(),
                    kind: NodeKind::Verification,
                    label: "verification".to_owned(),
                    payload: NodePayload::Verification {
                        mode: verification.mode.clone(),
                        status: verification.status.clone(),
                        formal_statement_digest: verification.formal_statement_digest.clone(),
                        failed_declarations: verification.failed_declarations.clone(),
                        verification_digest: verification_id.clone(),
                    },
                    edges: Vec::new(),
                },
            )?;
        }

        let mut declaration_drafts = BTreeMap::new();
        for declaration in &artifact.declarations {
            let statement_target = declaration
                .statement_digest
                .as_ref()
                .map(statement_node_id)
                .transpose()?;
            if let Some(id) = statement_target.as_ref() {
                insert_node(
                    &mut nodes_by_id,
                    MerkleNode {
                        id: id.clone(),
                        kind: NodeKind::Statement,
                        label: declaration
                            .statement_digest
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "null".to_owned()),
                        payload: NodePayload::Statement {
                            digest: declaration.statement_digest.clone(),
                        },
                        edges: Vec::new(),
                    },
                )?;
            }

            let body_target = declaration
                .body_digest
                .as_ref()
                .map(body_node_id)
                .transpose()?;
            if let Some(id) = body_target.as_ref() {
                insert_node(
                    &mut nodes_by_id,
                    MerkleNode {
                        id: id.clone(),
                        kind: NodeKind::Body,
                        label: declaration
                            .body_digest
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "null".to_owned()),
                        payload: NodePayload::Body {
                            digest: declaration.body_digest.clone(),
                        },
                        edges: Vec::new(),
                    },
                )?;
            }

            let mut dependencies = declaration.dependencies.clone();
            dependencies.sort();
            dependencies.dedup();
            declaration_drafts.insert(
                declaration.name.clone(),
                DeclarationDraft {
                    name: declaration.name.clone(),
                    kind: declaration.kind.clone(),
                    verification_status: declaration.verification_status.clone(),
                    statement: declaration.statement_digest.clone(),
                    body: declaration.body_digest.clone(),
                    dependencies,
                },
            );
        }

        let mut diagnostic_targets = Vec::new();
        for diagnostic in &artifact.diagnostics {
            let id = diagnostic_node_id(diagnostic)?;
            insert_node(
                &mut nodes_by_id,
                MerkleNode {
                    id: id.clone(),
                    kind: NodeKind::Diagnostic,
                    label: diagnostic
                        .code
                        .clone()
                        .unwrap_or_else(|| level_name(&diagnostic.level).to_owned()),
                    payload: NodePayload::Diagnostic {
                        level: diagnostic.level.clone(),
                        message: diagnostic.message.clone(),
                        code: diagnostic.code.clone(),
                    },
                    edges: Vec::new(),
                },
            )?;
            diagnostic_targets.push(id);
        }

        let mut declaration_cache = BTreeMap::new();
        let mut visiting = Vec::new();
        let declaration_names: Vec<_> = declaration_drafts.keys().cloned().collect();
        for name in declaration_names {
            declaration_node_id(
                &name,
                &declaration_drafts,
                &mut declaration_cache,
                &mut visiting,
                &mut nodes_by_id,
            )?;
        }

        let mut root_edges = vec![
            GraphEdge {
                label: EDGE_SOURCE.to_owned(),
                target: source_id,
            },
            GraphEdge {
                label: EDGE_ENVIRONMENT.to_owned(),
                target: environment_id,
            },
        ];
        if let Some(id) = verification_id {
            root_edges.push(GraphEdge {
                label: EDGE_VERIFICATION.to_owned(),
                target: id,
            });
        }
        root_edges.extend(declaration_cache.values().cloned().map(|target| GraphEdge {
            label: EDGE_DECLARATION.to_owned(),
            target,
        }));
        root_edges.extend(diagnostic_targets.into_iter().map(|target| GraphEdge {
            label: EDGE_DIAGNOSTIC.to_owned(),
            target,
        }));
        sort_edges(&mut root_edges);

        insert_node(
            &mut nodes_by_id,
            MerkleNode {
                id: root.clone(),
                kind: NodeKind::Artifact,
                label: "artifact".to_owned(),
                payload: NodePayload::Artifact {
                    artifact_id: root.clone(),
                },
                edges: root_edges,
            },
        )?;

        let mut nodes: Vec<_> = nodes_by_id.into_values().collect();
        nodes.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.label.cmp(&right.label))
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(Self {
            schema: GRAPH_SCHEMA_V0.to_owned(),
            root,
            nodes,
        })
    }

    pub fn to_dot(&self) -> String {
        let mut lines = Vec::new();
        lines.push("digraph axle {".to_owned());
        for node in &self.nodes {
            lines.push(format!(
                "  \"{}\" [label=\"{}\"];",
                node.id,
                dot_label(node)
            ));
        }
        for node in &self.nodes {
            for edge in &node.edges {
                lines.push(format!(
                    "  \"{}\" -> \"{}\" [label=\"{}\"];",
                    node.id, edge.target, edge.label
                ));
            }
        }
        lines.push("}".to_owned());
        lines.join("\n")
    }

    fn node_by_id(&self, id: &Digest) -> Option<&MerkleNode> {
        self.nodes.iter().find(|node| node.id == *id)
    }

    fn first_target(&self, label: &str) -> Option<Digest> {
        self.node_by_id(&self.root)
            .and_then(|root| root.edges.iter().find(|edge| edge.label == label))
            .map(|edge| edge.target.clone())
    }

    fn declaration_views(&self) -> Result<BTreeMap<String, DeclarationView>, GraphError> {
        let mut views = BTreeMap::new();
        for node in self
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, NodeKind::Declaration))
        {
            let NodePayload::Declaration {
                name,
                verification_status,
                ..
            } = &node.payload
            else {
                continue;
            };

            let statement_target = node
                .edges
                .iter()
                .find(|edge| edge.label == EDGE_STATEMENT)
                .map(|edge| edge.target.clone());
            let body_target = node
                .edges
                .iter()
                .find(|edge| edge.label == EDGE_BODY)
                .map(|edge| edge.target.clone());
            let mut dependency_targets: Vec<_> = node
                .edges
                .iter()
                .filter(|edge| edge.label == EDGE_DEPENDENCY)
                .map(|edge| edge.target.clone())
                .collect();
            dependency_targets.sort();

            views.insert(
                name.clone(),
                DeclarationView {
                    node_id: node.id.clone(),
                    name: name.clone(),
                    verification_status: verification_status.clone(),
                    statement_target,
                    body_target,
                    dependency_targets,
                },
            );
        }

        if views.values().any(|view| view.name.is_empty()) {
            return Err(GraphError::InvalidGraph(
                "declaration node missing name".to_owned(),
            ));
        }

        Ok(views)
    }
}

impl ArtifactDiff {
    pub fn between(old: &MerkleGraph, new: &MerkleGraph) -> Result<Self, GraphError> {
        let old_views = old.declaration_views()?;
        let new_views = new.declaration_views()?;

        let old_source = old
            .first_target(EDGE_SOURCE)
            .ok_or_else(|| GraphError::InvalidGraph("graph missing source edge".to_owned()))?;
        let new_source = new
            .first_target(EDGE_SOURCE)
            .ok_or_else(|| GraphError::InvalidGraph("graph missing source edge".to_owned()))?;
        let old_environment = old
            .first_target(EDGE_ENVIRONMENT)
            .ok_or_else(|| GraphError::InvalidGraph("graph missing environment edge".to_owned()))?;
        let new_environment = new
            .first_target(EDGE_ENVIRONMENT)
            .ok_or_else(|| GraphError::InvalidGraph("graph missing environment edge".to_owned()))?;
        let old_verification = old.first_target(EDGE_VERIFICATION);
        let new_verification = new.first_target(EDGE_VERIFICATION);

        let old_names: BTreeSet<_> = old_views.keys().cloned().collect();
        let new_names: BTreeSet<_> = new_views.keys().cloned().collect();

        let added_declarations: Vec<_> = new_names.difference(&old_names).cloned().collect();
        let removed_declarations: Vec<_> = old_names.difference(&new_names).cloned().collect();

        let mut changed_declarations = Vec::new();
        for name in old_names.intersection(&new_names) {
            let old_view = old_views
                .get(name)
                .expect("declaration should exist in old graph");
            let new_view = new_views
                .get(name)
                .expect("declaration should exist in new graph");

            let statement_changed = old_view.statement_target != new_view.statement_target;
            let body_changed = old_view.body_target != new_view.body_target;
            let verification_status_changed =
                old_view.verification_status != new_view.verification_status;
            let dependencies_changed = old_view.dependency_targets != new_view.dependency_targets;

            if old_view.node_id != new_view.node_id
                || statement_changed
                || body_changed
                || verification_status_changed
                || dependencies_changed
            {
                changed_declarations.push(ChangedDeclaration {
                    name: name.clone(),
                    old_node_id: old_view.node_id.clone(),
                    new_node_id: new_view.node_id.clone(),
                    statement_changed,
                    body_changed,
                    verification_status_changed,
                    dependencies_changed,
                });
            }
        }

        let source_changed = old_source != new_source;
        let environment_changed = old_environment != new_environment;
        let verification_changed = old_verification != new_verification;
        let identical = old.root == new.root
            && !source_changed
            && !environment_changed
            && !verification_changed
            && added_declarations.is_empty()
            && removed_declarations.is_empty()
            && changed_declarations.is_empty();

        Ok(Self {
            schema: DIFF_SCHEMA_V0.to_owned(),
            identical,
            old_root: old.root.clone(),
            new_root: new.root.clone(),
            old_source,
            new_source,
            source_changed,
            old_environment,
            new_environment,
            environment_changed,
            old_verification,
            new_verification,
            verification_changed,
            added_declarations,
            removed_declarations,
            changed_declarations,
        })
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("root: {}", change_word(self.old_root != self.new_root)),
            format!("old_root: {}", self.old_root),
            format!("new_root: {}", self.new_root),
            format!("source: {}", change_word(self.source_changed)),
            format!("environment: {}", change_word(self.environment_changed)),
            format!("verification: {}", change_word(self.verification_changed)),
        ];

        if self.added_declarations.is_empty() {
            lines.push("added_declarations: none".to_owned());
        } else {
            lines.push(format!(
                "added_declarations: {}",
                self.added_declarations.len()
            ));
            for name in &self.added_declarations {
                lines.push(format!("added_declaration: {name}"));
            }
        }

        if self.removed_declarations.is_empty() {
            lines.push("removed_declarations: none".to_owned());
        } else {
            lines.push(format!(
                "removed_declarations: {}",
                self.removed_declarations.len()
            ));
            for name in &self.removed_declarations {
                lines.push(format!("removed_declaration: {name}"));
            }
        }

        if self.changed_declarations.is_empty() {
            lines.push("changed_declarations: none".to_owned());
        } else {
            lines.push(format!(
                "changed_declarations: {}",
                self.changed_declarations.len()
            ));
            for change in &self.changed_declarations {
                let mut parts = Vec::new();
                if change.statement_changed {
                    parts.push("statement");
                }
                if change.body_changed {
                    parts.push("body");
                }
                if change.verification_status_changed {
                    parts.push("verification_status");
                }
                if change.dependencies_changed {
                    parts.push("dependencies");
                }
                lines.push(format!(
                    "changed_declaration: {} [{}]",
                    change.name,
                    parts.join(", ")
                ));
            }
        }

        lines.join("\n")
    }
}

fn change_word(changed: bool) -> &'static str {
    if changed { "changed" } else { "unchanged" }
}

fn insert_node(
    nodes: &mut BTreeMap<Digest, MerkleNode>,
    node: MerkleNode,
) -> Result<(), GraphError> {
    match nodes.get(&node.id) {
        Some(existing) if existing == &node => Ok(()),
        Some(_) => Err(GraphError::DuplicateNode(node.id)),
        None => {
            nodes.insert(node.id.clone(), node);
            Ok(())
        }
    }
}

fn statement_node_id(digest: &Digest) -> Result<Digest, GraphError> {
    Ok(Digest::from_canonical_json(&NodePayload::Statement {
        digest: Some(digest.clone()),
    })?)
}

fn body_node_id(digest: &Digest) -> Result<Digest, GraphError> {
    Ok(Digest::from_canonical_json(&NodePayload::Body {
        digest: Some(digest.clone()),
    })?)
}

fn diagnostic_node_id(diagnostic: &Diagnostic) -> Result<Digest, GraphError> {
    Ok(Digest::from_canonical_json(&NodePayload::Diagnostic {
        level: diagnostic.level.clone(),
        message: diagnostic.message.clone(),
        code: diagnostic.code.clone(),
    })?)
}

fn declaration_node_id(
    name: &str,
    drafts: &BTreeMap<String, DeclarationDraft>,
    cache: &mut BTreeMap<String, Digest>,
    visiting: &mut Vec<String>,
    nodes: &mut BTreeMap<Digest, MerkleNode>,
) -> Result<Digest, GraphError> {
    if let Some(id) = cache.get(name) {
        return Ok(id.clone());
    }
    if let Some(index) = visiting.iter().position(|current| current == name) {
        let mut cycle = visiting[index..].to_vec();
        cycle.push(name.to_owned());
        return Err(GraphError::CyclicDependency(cycle.join(" -> ")));
    }

    let draft = drafts
        .get(name)
        .ok_or_else(|| GraphError::InvalidGraph(format!("missing declaration draft for {name}")))?;
    visiting.push(name.to_owned());

    let mut edges = Vec::new();
    if let Some(statement) = draft.statement.as_ref() {
        edges.push(GraphEdge {
            label: EDGE_STATEMENT.to_owned(),
            target: statement_node_id(statement)?,
        });
    }
    if let Some(body) = draft.body.as_ref() {
        edges.push(GraphEdge {
            label: EDGE_BODY.to_owned(),
            target: body_node_id(body)?,
        });
    }

    for dependency in &draft.dependencies {
        if !drafts.contains_key(dependency) {
            return Err(GraphError::MissingDependency {
                declaration: draft.name.clone(),
                dependency: dependency.clone(),
            });
        }
        let target = declaration_node_id(dependency, drafts, cache, visiting, nodes)?;
        edges.push(GraphEdge {
            label: EDGE_DEPENDENCY.to_owned(),
            target,
        });
    }
    sort_edges(&mut edges);

    let payload = NodePayload::Declaration {
        name: draft.name.clone(),
        declaration_kind: draft.kind.clone(),
        verification_status: draft.verification_status.clone(),
    };
    let node_id = Digest::from_canonical_json(&DeclarationNodeHashInput {
        kind: NodeKind::Declaration,
        label: draft.name.clone(),
        payload: &payload,
        edges: &edges,
    })?;
    insert_node(
        nodes,
        MerkleNode {
            id: node_id.clone(),
            kind: NodeKind::Declaration,
            label: draft.name.clone(),
            payload,
            edges,
        },
    )?;

    visiting.pop();
    cache.insert(name.to_owned(), node_id.clone());
    Ok(node_id)
}

fn sort_edges(edges: &mut Vec<GraphEdge>) {
    edges.sort_by(|left, right| {
        left.label
            .cmp(&right.label)
            .then_with(|| left.target.cmp(&right.target))
    });
}

fn digest_environment(environment: &Environment) -> Result<Digest, GraphError> {
    let mut normalized = environment.clone();
    normalized.environment_digest = None;
    Ok(Digest::from_canonical_json(&normalized)?)
}

fn dot_label(node: &MerkleNode) -> String {
    let title = match node.kind {
        NodeKind::Artifact => "artifact",
        NodeKind::Source => "source",
        NodeKind::Environment => "environment",
        NodeKind::Verification => "verification",
        NodeKind::Declaration => "declaration",
        NodeKind::Statement => "statement",
        NodeKind::Body => "body",
        NodeKind::Diagnostic => "diagnostic",
    };
    let detail = match &node.payload {
        NodePayload::Artifact { artifact_id } => artifact_id.to_string(),
        NodePayload::Source { path, .. } => path.clone().unwrap_or_else(|| "source".to_owned()),
        NodePayload::Environment { lean_version, .. } => lean_version
            .clone()
            .unwrap_or_else(|| "environment".to_owned()),
        NodePayload::Verification { status, .. } => match status {
            VerificationResultStatus::Pass => "pass".to_owned(),
            VerificationResultStatus::Fail => "fail".to_owned(),
        },
        NodePayload::Declaration { name, .. } => name.clone(),
        NodePayload::Statement { digest } | NodePayload::Body { digest } => digest
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "null".to_owned()),
        NodePayload::Diagnostic { code, level, .. } => {
            code.clone().unwrap_or_else(|| level_name(level).to_owned())
        }
    };

    format!("{title}\\n{detail}")
}

fn level_name(level: &DiagnosticLevel) -> &'static str {
    match level {
        DiagnosticLevel::Info => "info",
        DiagnosticLevel::Warning => "warning",
        DiagnosticLevel::Error => "error",
    }
}

#[derive(Serialize)]
struct DeclarationNodeHashInput<'a> {
    kind: NodeKind,
    label: String,
    payload: &'a NodePayload,
    edges: &'a [GraphEdge],
}

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("failed to hash graph node content: {0}")]
    Hash(#[from] HashError),
    #[error("failed to compute artifact digest: {0}")]
    Artifact(#[from] anyhow::Error),
    #[error(
        "unresolved local declaration dependency '{dependency}' referenced by declaration '{declaration}'"
    )]
    MissingDependency {
        declaration: String,
        dependency: String,
    },
    #[error("cyclic local declaration dependency graph: {0}")]
    CyclicDependency(String),
    #[error("graph derivation produced conflicting node data for {0}")]
    DuplicateNode(Digest),
    #[error("invalid graph: {0}")]
    InvalidGraph(String),
}

#[cfg(test)]
mod tests {
    use super::{ArtifactDiff, MerkleGraph};
    use axle_core::{
        AdapterMetadata, AxleArtifact, Declaration, DeclarationKind, Diagnostic, DiagnosticLevel,
        VerificationMode, VerificationResultStatus, VerificationStatus, VerificationSummary,
    };
    use axle_hash::Digest;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn sample_artifact() -> AxleArtifact {
        let mut artifact = AxleArtifact::new_v0();
        artifact.source.module = Some("Sample".to_owned());
        artifact.source.path = Some("Sample.lean".to_owned());
        artifact.source.source_text = Some("theorem Sample.bar : True := trivial".to_owned());
        artifact.manifest.environment.lean_version = Some("lean-4.28.0".to_owned());
        artifact.declarations = vec![
            Declaration {
                name: "Sample.foo".to_owned(),
                kind: DeclarationKind::Def,
                statement_digest: Some(Digest::sha256("Nat")),
                body_digest: Some(Digest::sha256("1")),
                dependencies: Vec::new(),
                verification_status: VerificationStatus::Verified,
            },
            Declaration {
                name: "Sample.bar".to_owned(),
                kind: DeclarationKind::Theorem,
                statement_digest: Some(Digest::sha256("Sample.foo = 1")),
                body_digest: Some(Digest::sha256("rfl")),
                dependencies: vec!["Sample.foo".to_owned()],
                verification_status: VerificationStatus::Verified,
            },
        ];
        artifact.diagnostics = vec![Diagnostic {
            level: DiagnosticLevel::Warning,
            message: "sample warning".to_owned(),
            code: Some("graph.test".to_owned()),
        }];
        artifact
    }

    #[test]
    fn derives_graph_for_build_artifact_without_verification() {
        let graph = MerkleGraph::derive(&sample_artifact()).unwrap();

        assert_eq!(graph.schema, super::GRAPH_SCHEMA_V0);
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| matches!(node.kind, super::NodeKind::Artifact))
        );
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| matches!(node.kind, super::NodeKind::Declaration))
        );
        assert!(
            !graph
                .nodes
                .iter()
                .any(|node| matches!(node.kind, super::NodeKind::Verification))
        );
    }

    #[test]
    fn derives_graph_for_verified_artifact_with_verification_summary() {
        let mut artifact = sample_artifact();
        artifact.verification = Some(VerificationSummary {
            mode: VerificationMode::VerifyProof,
            status: VerificationResultStatus::Pass,
            formal_statement_digest: Digest::sha256("theorem Sample.bar : Sample.foo = 1"),
            failed_declarations: Vec::new(),
        });

        let graph = MerkleGraph::derive(&artifact).unwrap();

        assert!(
            graph
                .nodes
                .iter()
                .any(|node| matches!(node.kind, super::NodeKind::Verification))
        );
    }

    #[test]
    fn graph_is_stable_when_dependency_order_changes() {
        let left = sample_artifact();
        let mut right = sample_artifact();
        right.declarations[1].dependencies = vec!["Sample.foo".to_owned(), "Sample.foo".to_owned()];
        let left_graph = MerkleGraph::derive(&left).unwrap();
        let right_graph = MerkleGraph::derive(&right).unwrap();
        let left_bar = left_graph
            .nodes
            .iter()
            .find(|node| node.label == "Sample.bar")
            .unwrap();
        let right_bar = right_graph
            .nodes
            .iter()
            .find(|node| node.label == "Sample.bar")
            .unwrap();

        assert_eq!(left_bar.id, right_bar.id);
        assert_eq!(left_bar.edges, right_bar.edges);
    }

    #[test]
    fn adapter_metadata_does_not_change_derived_graph() {
        let left = sample_artifact();
        let mut right = sample_artifact();
        let mut requests = BTreeMap::new();
        requests.insert("check".to_owned(), json!({ "content": "foo" }));
        let mut responses = BTreeMap::new();
        responses.insert("check".to_owned(), json!({ "timings": { "total_ms": 10 } }));
        right.adapter = Some(AdapterMetadata::build(requests, responses));

        assert_eq!(
            MerkleGraph::derive(&left).unwrap(),
            MerkleGraph::derive(&right).unwrap()
        );
    }

    #[test]
    fn unresolved_dependency_fails_with_clear_error() {
        let mut artifact = sample_artifact();
        artifact.declarations[1].dependencies = vec!["Sample.missing".to_owned()];

        let error = MerkleGraph::derive(&artifact).unwrap_err().to_string();
        assert!(error.contains("unresolved local declaration dependency 'Sample.missing'"));
    }

    #[test]
    fn diff_identical_artifacts_is_empty() {
        let graph = MerkleGraph::derive(&sample_artifact()).unwrap();
        let diff = ArtifactDiff::between(&graph, &graph).unwrap();

        assert!(diff.identical);
        assert!(diff.changed_declarations.is_empty());
        assert!(diff.added_declarations.is_empty());
        assert!(diff.removed_declarations.is_empty());
    }

    #[test]
    fn diff_detects_added_and_removed_declarations() {
        let old = sample_artifact();
        let mut new = sample_artifact();
        new.declarations.remove(0);
        new.declarations[0].dependencies = Vec::new();
        new.declarations.push(Declaration {
            name: "Sample.baz".to_owned(),
            kind: DeclarationKind::Lemma,
            statement_digest: Some(Digest::sha256("True")),
            body_digest: Some(Digest::sha256("trivial")),
            dependencies: Vec::new(),
            verification_status: VerificationStatus::Verified,
        });

        let diff = ArtifactDiff::between(
            &MerkleGraph::derive(&old).unwrap(),
            &MerkleGraph::derive(&new).unwrap(),
        )
        .unwrap();

        assert_eq!(diff.added_declarations, vec!["Sample.baz".to_owned()]);
        assert_eq!(diff.removed_declarations, vec!["Sample.foo".to_owned()]);
    }

    #[test]
    fn diff_detects_statement_body_status_and_dependency_changes() {
        let old = sample_artifact();
        let mut new = sample_artifact();
        new.declarations[1].statement_digest = Some(Digest::sha256("Sample.foo = 2"));
        new.declarations[1].body_digest = Some(Digest::sha256("by omega"));
        new.declarations[1].verification_status = VerificationStatus::Failed;
        new.declarations[1].dependencies = Vec::new();

        let diff = ArtifactDiff::between(
            &MerkleGraph::derive(&old).unwrap(),
            &MerkleGraph::derive(&new).unwrap(),
        )
        .unwrap();

        assert_eq!(diff.changed_declarations.len(), 1);
        let change = &diff.changed_declarations[0];
        assert!(change.statement_changed);
        assert!(change.body_changed);
        assert!(change.verification_status_changed);
        assert!(change.dependencies_changed);
    }

    #[test]
    fn diff_detects_verification_changes_without_declaration_changes() {
        let old = sample_artifact();
        let mut new = sample_artifact();
        new.verification = Some(VerificationSummary {
            mode: VerificationMode::VerifyProof,
            status: VerificationResultStatus::Fail,
            formal_statement_digest: Digest::sha256("theorem Sample.bar : True"),
            failed_declarations: vec!["Sample.bar".to_owned()],
        });

        let diff = ArtifactDiff::between(
            &MerkleGraph::derive(&old).unwrap(),
            &MerkleGraph::derive(&new).unwrap(),
        )
        .unwrap();

        assert!(diff.verification_changed);
        assert!(diff.changed_declarations.is_empty());
    }

    #[test]
    fn graph_json_matches_golden_fixture() {
        let graph = MerkleGraph::derive(&sample_artifact()).unwrap();
        let actual = serde_json::to_string_pretty(&graph).unwrap();
        let expected = include_str!("../../../tests/golden/graph-build.json").trim();

        assert_eq!(actual, expected);
    }

    #[test]
    fn diff_text_matches_golden_fixture() {
        let old = sample_artifact();
        let mut new = sample_artifact();
        new.declarations[1].body_digest = Some(Digest::sha256("by omega"));
        let diff = ArtifactDiff::between(
            &MerkleGraph::derive(&old).unwrap(),
            &MerkleGraph::derive(&new).unwrap(),
        )
        .unwrap();
        let actual = diff.to_text();
        let expected = include_str!("../../../tests/golden/diff-body-change.txt").trim();

        assert_eq!(actual, expected);
    }
}
