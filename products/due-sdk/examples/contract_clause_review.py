"""DUE SDK demo: AI-assisted contract clause review.

Run from `products/due-sdk` with:

    python examples/contract_clause_review.py

This example is intentionally fictional and demo-grade.
"""

from __future__ import annotations

from due import authority, bundle, disclosure, duty, matter, privilege, proof, reliance, review


def main() -> None:
    m = matter.bind(
        matter_id="M-2026-0142",
        client="Acme Robotics",
        matter_type="vendor_contract_review",
        jurisdiction="Delaware",
        confidentiality="attorney_client",
    )

    action = duty.record_action(
        matter=m,
        action_type="ai_clause_review",
        actor="contract_review_agent_v1",
        issue="indemnity_clause_risk",
        output="Flagged broad indemnity language requiring attorney review.",
    )

    authority_event = authority.record(
        action=action,
        authority_type="engagement_scope",
        source="Outside counsel engagement letter",
        rule="AI may assist review, but final legal judgment requires attorney approval.",
    )

    privilege_event = privilege.record(
        action=action,
        status="privileged",
        basis="Attorney-client legal advice workflow",
        reviewer="jane.lawyer@example.com",
    )

    disclosure_event = disclosure.record(
        action=action,
        status="internal_work_product",
        audience="legal_team_only",
        note="Not sent to client or counterparty before attorney review.",
    )

    review_event = review.record(
        action=action,
        reviewer="jane.lawyer@example.com",
        role="licensed_attorney",
        decision="approved_with_revision",
        note="Attorney confirmed the indemnity risk and revised the recommendation.",
    )

    reliance_event = reliance.record(
        action=action,
        relied_upon_by_human=True,
        reliance_type="attorney_reviewed_not_auto_sent",
        final_decision_maker="jane.lawyer@example.com",
    )

    proof_event = proof.attach_axle_receipt(
        action=action,
        artifact_id="sha256:example-artifact-digest",
        receipt_id="sha256:example-axle-receipt-digest",
        policy_id="attorney_review_required_before_external_reliance",
        verification_status="pass",
        note="Demo reference to an AXLE receipt for a formalized workflow constraint.",
    )

    legal_bundle = bundle.export(
        matter=m,
        actions=[action],
        authorities=[authority_event],
        privileges=[privilege_event],
        disclosures=[disclosure_event],
        reviews=[review_event],
        reliances=[reliance_event],
        proof_artifacts=[proof_event],
    )

    print(legal_bundle.to_markdown())


if __name__ == "__main__":
    main()
