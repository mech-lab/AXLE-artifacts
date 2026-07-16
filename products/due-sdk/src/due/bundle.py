"""Legal Defensibility Bundle export for DUE SDK."""

from __future__ import annotations

from .models import (
    AuthorityRecord,
    AxleReceiptReference,
    DisclosureRecord,
    DutyAction,
    LegalDefensibilityBundle,
    MatterContext,
    PrivilegeRecord,
    RelianceRecord,
    ReviewRecord,
    stable_id,
)


def export(
    *,
    matter: MatterContext,
    actions: list[DutyAction],
    authorities: list[AuthorityRecord] | None = None,
    privileges: list[PrivilegeRecord] | None = None,
    disclosures: list[DisclosureRecord] | None = None,
    reviews: list[ReviewRecord] | None = None,
    reliances: list[RelianceRecord] | None = None,
    proof_artifacts: list[AxleReceiptReference] | None = None,
) -> LegalDefensibilityBundle:
    """Export a matter-bound Legal Defensibility Bundle.

    The bundle is a structured evidence package, not a legal conclusion.
    """
    authorities = authorities or []
    privileges = privileges or []
    disclosures = disclosures or []
    reviews = reviews or []
    reliances = reliances or []
    proof_artifacts = proof_artifacts or []

    seed = {
        "matter_id": matter.matter_id,
        "actions": [a.action_id for a in actions],
        "authorities": [a.authority_id for a in authorities],
        "privileges": [p.privilege_id for p in privileges],
        "disclosures": [d.disclosure_id for d in disclosures],
        "reviews": [r.review_id for r in reviews],
        "reliances": [r.reliance_id for r in reliances],
        "proof_artifacts": [p.proof_id for p in proof_artifacts],
    }

    return LegalDefensibilityBundle(
        schema="due.legal_defensibility_bundle.v1",
        bundle_id=stable_id("ldb", seed),
        matter=matter,
        actions=actions,
        authorities=authorities,
        privileges=privileges,
        disclosures=disclosures,
        reviews=reviews,
        reliances=reliances,
        proof_artifacts=proof_artifacts,
    )
