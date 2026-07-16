# DUE SDK

## Experimental Python SDK for legaltech developers building duty-bound AI workflows.

DUE SDK is an experimental product slice inside `AXLE-artifacts`. It is for legaltech developers, including lawyers who code in Python, who need to prototype evidence records for AI-assisted legal workflows.

Bluntly: this is not production legal infrastructure yet. It is a demo-grade SDK for exploring how AI-assisted legal actions can be recorded, packaged, and connected to AXLE proof artifacts.

DUE SDK helps developers generate **Duty Receipts** and export **Legal Defensibility Bundles** that connect AI-assisted actions to matter context, authority, privilege, disclosure, human review, reliance, and optional AXLE-backed proof artifacts.

## What this is

DUE SDK is:

- a Python scaffold for legaltech developers
- an experimental receipt model for AI-assisted legal actions
- a way to bind legal workflow facts to AXLE artifact references
- a bundle exporter for legal defensibility demos
- a bridge between `.axle` proof artifacts and lawyer-readable evidence records

## What this is not

DUE SDK is not:

- legal advice
- a legal correctness engine
- a privilege determination system
- an admissibility guarantee
- a substitute for attorney judgment
- a production-grade compliance platform
- a claim that Lean or AXLE can prove a legal conclusion correct

DUE records workflow evidence. It does not decide the law.

## Target user

The first target user is a legaltech developer who may also be a lawyer.

They likely understand:

- matter context
- privilege
- disclosure
- authority
- reliance
- attorney review
- professional duties
- litigation risk

They may also write enough Python to build prototypes, internal tools, legal AI agents, or review workflows.

The SDK should therefore be readable to both sides of that person: the lawyer and the developer.

## Product thesis

AI will increasingly assist legal workflows: contract review, disclosure review, research, intake, privilege review, litigation hold, compliance review, claims review, and internal legal operations.

The hard question is not whether the AI is impressive. The hard question is:

> Can the team show what the AI-assisted action was, who reviewed it, what authority governed it, what privilege or disclosure posture applied, whether anyone relied on it, and which machine-checkable workflow constraints were satisfied?

DUE SDK creates a structured record for that question.

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

That separation is intentional:

- AXLE handles formal or machine-checkable constraints.
- AXLE-artifacts preserves proof outputs as durable artifacts.
- AXLE Receipts bind claims to artifact digests.
- DUE records legal workflow context around those artifacts.

## Demo use case

The first demo models an AI-assisted contract clause review.

A legaltech startup uses an AI review agent to flag a risky indemnity clause in a vendor agreement. DUE records:

1. the legal matter
2. the AI-assisted clause review action
3. the authority basis for using AI assistance
4. the privilege and disclosure posture
5. the attorney human review step
6. the reliance posture
7. the AXLE receipt reference for a machine-checkable workflow policy
8. the exported Legal Defensibility Bundle

Example policy constraint:

```text
AI-generated contract risk output cannot be externally relied upon until a licensed attorney review record exists.
```

AXLE may verify a formalized version of that workflow constraint. DUE records the legal workflow facts and references the AXLE receipt.

## Python quickstart

```python
from due import authority, bundle, disclosure, duty, matter, privilege, proof, reliance, review

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
```

## Core concepts

| Concept | Meaning |
|---|---|
| Matter Context | The legal matter, client, jurisdiction, and confidentiality posture. |
| Duty Receipt | A structured record of an AI-assisted action performed inside a duty-bound workflow. |
| Authority Record | The policy, rule, engagement scope, or human instruction authorizing the action. |
| Privilege Record | The privilege or confidentiality posture asserted for the action. |
| Disclosure Record | Whether and how an AI-assisted output was disclosed. |
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

DUE speaks legal workflow language. AXLE speaks formal proof language. The bundle connects them without pretending they are the same thing.

Use DUE for:

- legaltech demos
- lawyers-who-code prototypes
- duty-bound AI workflow receipts
- matter-bound defensibility records
- proof-backed workflow constraints
- legal AI evidence exports

Do not use DUE to claim:

- mathematical proof of legal correctness
- replacement of attorney judgment
- automatic admissibility
- legal advice generation
- privilege determination without human review
- production readiness

## Current status

Experimental. Demo-grade. Intended for rapid iteration.

The current goal is to make the DUE legal-defensibility concept concrete enough to test with legaltech developers, not to ship a regulated legal product.
