# DUE SDK

## Receipts for duty-bound AI.

DUE SDK is a Python-first developer product slice for legaltech teams building AI-assisted legal workflows. It helps lawyers who code generate **Duty Receipts** and export **Legal Defensibility Bundles** that connect AI-assisted actions to matter context, authority, privilege, disclosure, human review, reliance, and optional AXLE-backed proof artifacts.

DUE does **not** claim to prove that a legal conclusion is correct. It records what happened, what duty was implicated, what authority governed the action, who reviewed it, and which machine-checkable workflow constraints were satisfied.

## Why this lives in AXLE-artifacts

AXLE-artifacts freezes AXLE-compatible Lean outputs into durable `.axle` artifacts. DUE SDK sits one layer above that:

```text
AXLE / Lean verification
  ↓
.axle artifact
  ↓
AXLE Receipt
  ↓
DUE Duty Receipt
  ↓
Legal Defensibility Bundle
```

An AXLE Receipt binds a formal or policy verification claim to a specific `.axle` artifact digest. A DUE Duty Receipt binds that proof evidence to legal workflow context.

## Demo use case

The first demo models an AI-assisted contract clause review:

1. Bind a legal matter.
2. Record an AI-assisted clause review action.
3. Record the authority basis for using AI assistance.
4. Record privilege and disclosure posture.
5. Record attorney human review.
6. Attach an AXLE receipt for a machine-checkable policy constraint.
7. Export a Legal Defensibility Bundle.

## Python quickstart

```python
from due import bundle, duty, matter, proof, review

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

review_event = review.record(
    action=action,
    reviewer="jane.lawyer@example.com",
    role="licensed_attorney",
    decision="approved_with_revision",
    note="Attorney confirmed the indemnity risk and revised the recommendation.",
)

proof_event = proof.attach_axle_receipt(
    action=action,
    artifact_id="sha256:example-artifact-digest",
    receipt_id="sha256:example-axle-receipt-digest",
    policy_id="attorney_review_required_before_external_reliance",
    verification_status="pass",
)

legal_bundle = bundle.export(
    matter=m,
    actions=[action],
    reviews=[review_event],
    proof_artifacts=[proof_event],
)

print(legal_bundle.to_markdown())
```

## Core concepts

| Concept | Meaning |
|---|---|
| Matter Context | The legal matter, client, jurisdiction, and confidentiality posture. |
| Duty Receipt | A structured record of an AI-assisted action performed inside a duty-bound workflow. |
| Authority Record | The policy, rule, engagement scope, or human instruction authorizing the action. |
| Review Record | Evidence that a qualified human reviewed or approved the action. |
| Reliance Record | Evidence of whether the AI output was relied on, externally sent, or only used internally. |
| AXLE Receipt | A proof-artifact receipt binding a formal verification claim to a `.axle` artifact digest. |
| Legal Defensibility Bundle | A matter-level evidence package combining action, authority, review, privilege, reliance, disclosure, and proof artifacts. |

## Initial API surface

```python
due.matter.bind(...)
due.duty.record_action(...)
due.authority.record(...)
due.privilege.record(...)
due.disclosure.record(...)
due.review.record(...)
due.reliance.record(...)
due.proof.attach_axle_receipt(...)
due.bundle.export(...)
```

## Design rule

DUE speaks legal language. AXLE speaks formal proof language. The bundle connects them without collapsing them.

Use DUE for:

- duty-bound AI workflows
- legaltech demos
- matter-bound defensibility records
- proof-backed workflow constraints
- legal AI evidence exports

Avoid claiming:

- mathematical proof of legal correctness
- replacement of attorney judgment
- automatic admissibility
- legal advice generation
- privilege determination without human review
