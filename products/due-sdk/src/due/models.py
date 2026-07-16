"""Core DUE SDK data model.

The model is intentionally small and JSON-native. It records legal workflow
facts and references AXLE receipts, but it does not attempt to prove legal
correctness.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from hashlib import sha256
import json
from typing import Any, Literal
from uuid import uuid4

JsonDict = dict[str, Any]


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def canonical_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def digest(value: Any) -> str:
    return "sha256:" + sha256(canonical_json(value).encode("utf-8")).hexdigest()


def stable_id(prefix: str, payload: Any) -> str:
    return f"{prefix}_{digest(payload).split(':', 1)[1][:24]}"


@dataclass(frozen=True)
class MatterContext:
    schema: str
    matter_id: str
    client: str
    matter_type: str
    jurisdiction: str
    confidentiality: str
    created_at: str = field(default_factory=utc_now)

    def to_dict(self) -> JsonDict:
        return asdict(self)


@dataclass(frozen=True)
class DutyAction:
    schema: str
    action_id: str
    matter_id: str
    action_type: str
    actor: str
    issue: str
    output: str
    created_at: str = field(default_factory=utc_now)

    def to_dict(self) -> JsonDict:
        return asdict(self)


@dataclass(frozen=True)
class AuthorityRecord:
    schema: str
    authority_id: str
    action_id: str
    authority_type: str
    source: str
    rule: str
    created_at: str = field(default_factory=utc_now)

    def to_dict(self) -> JsonDict:
        return asdict(self)


@dataclass(frozen=True)
class PrivilegeRecord:
    schema: str
    privilege_id: str
    action_id: str
    status: str
    basis: str
    reviewer: str | None = None
    created_at: str = field(default_factory=utc_now)

    def to_dict(self) -> JsonDict:
        return asdict(self)


@dataclass(frozen=True)
class DisclosureRecord:
    schema: str
    disclosure_id: str
    action_id: str
    status: str
    audience: str
    note: str | None = None
    created_at: str = field(default_factory=utc_now)

    def to_dict(self) -> JsonDict:
        return asdict(self)


@dataclass(frozen=True)
class ReviewRecord:
    schema: str
    review_id: str
    action_id: str
    reviewer: str
    role: str
    decision: str
    note: str | None = None
    created_at: str = field(default_factory=utc_now)

    def to_dict(self) -> JsonDict:
        return asdict(self)


@dataclass(frozen=True)
class RelianceRecord:
    schema: str
    reliance_id: str
    action_id: str
    relied_upon_by_human: bool
    reliance_type: str
    final_decision_maker: str | None = None
    created_at: str = field(default_factory=utc_now)

    def to_dict(self) -> JsonDict:
        return asdict(self)


@dataclass(frozen=True)
class AxleReceiptReference:
    schema: str
    proof_id: str
    action_id: str
    artifact_id: str
    receipt_id: str
    policy_id: str
    verification_status: Literal["pass", "fail", "unknown"]
    note: str | None = None
    created_at: str = field(default_factory=utc_now)

    def to_dict(self) -> JsonDict:
        return asdict(self)


@dataclass(frozen=True)
class LegalDefensibilityBundle:
    schema: str
    bundle_id: str
    matter: MatterContext
    actions: list[DutyAction]
    authorities: list[AuthorityRecord] = field(default_factory=list)
    privileges: list[PrivilegeRecord] = field(default_factory=list)
    disclosures: list[DisclosureRecord] = field(default_factory=list)
    reviews: list[ReviewRecord] = field(default_factory=list)
    reliances: list[RelianceRecord] = field(default_factory=list)
    proof_artifacts: list[AxleReceiptReference] = field(default_factory=list)
    created_at: str = field(default_factory=utc_now)

    def to_dict(self) -> JsonDict:
        return {
            "schema": self.schema,
            "bundle_id": self.bundle_id,
            "created_at": self.created_at,
            "matter": self.matter.to_dict(),
            "actions": [a.to_dict() for a in self.actions],
            "authorities": [a.to_dict() for a in self.authorities],
            "privileges": [p.to_dict() for p in self.privileges],
            "disclosures": [d.to_dict() for d in self.disclosures],
            "reviews": [r.to_dict() for r in self.reviews],
            "reliances": [r.to_dict() for r in self.reliances],
            "proof_artifacts": [p.to_dict() for p in self.proof_artifacts],
        }

    def digest(self) -> str:
        body = self.to_dict() | {"bundle_id": None}
        return digest(body)

    def to_json(self, *, indent: int = 2) -> str:
        return json.dumps(self.to_dict(), indent=indent, sort_keys=True, ensure_ascii=False)

    def to_markdown(self) -> str:
        proof_lines = [
            f"- `{p.policy_id}` — {p.verification_status} ({p.receipt_id})"
            for p in self.proof_artifacts
        ] or ["- No AXLE receipt references attached."]

        review_lines = [
            f"- {r.reviewer} ({r.role}) — {r.decision}"
            for r in self.reviews
        ] or ["- No human review records attached."]

        action_lines = [
            f"- `{a.action_type}` by `{a.actor}`: {a.issue}"
            for a in self.actions
        ]

        return "\n".join(
            [
                "# Legal Defensibility Bundle",
                "",
                f"**Bundle ID:** `{self.bundle_id}`",
                f"**Bundle Digest:** `{self.digest()}`",
                f"**Matter:** `{self.matter.matter_id}`",
                f"**Client:** {self.matter.client}",
                f"**Jurisdiction:** {self.matter.jurisdiction}",
                f"**Confidentiality:** {self.matter.confidentiality}",
                "",
                "## AI-Assisted Actions",
                *action_lines,
                "",
                "## Human Review",
                *review_lines,
                "",
                "## AXLE Receipt References",
                *proof_lines,
                "",
                "## Defensibility Note",
                "This bundle records matter-bound workflow evidence. It does not assert legal correctness, replace attorney judgment, or determine admissibility.",
            ]
        )


def make_id(prefix: str) -> str:
    return f"{prefix}_{uuid4().hex[:16]}"
