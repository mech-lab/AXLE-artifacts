# Legal Defensibility Bundle

**Status:** experimental demo fixture  
**Product slice:** DUE SDK  
**Audience:** legaltech developers and lawyers who code in Python

## Matter

| Field | Value |
|---|---|
| Matter ID | `M-2026-0142` |
| Client | Acme Robotics |
| Matter Type | Vendor contract review |
| Jurisdiction | Delaware |
| Confidentiality | Attorney-client |

## AI-Assisted Action

| Field | Value |
|---|---|
| Action Type | `ai_clause_review` |
| Actor | `contract_review_agent_v1` |
| Issue | Indemnity clause risk |
| Output | Flagged broad indemnity language requiring attorney review. |

## Authority

The action was performed under an engagement-scope rule:

> AI may assist review, but final legal judgment requires attorney approval.

## Privilege And Disclosure Posture

| Field | Value |
|---|---|
| Privilege Status | Privileged |
| Basis | Attorney-client legal advice workflow |
| Disclosure Status | Internal work product |
| Audience | Legal team only |

## Human Review

| Reviewer | Role | Decision |
|---|---|---|
| `jane.lawyer@example.com` | Licensed attorney | Approved with revision |

## Reliance

The AI output was not externally relied upon before attorney review. Final decision-maker: `jane.lawyer@example.com`.

## AXLE Receipt Reference

| Field | Value |
|---|---|
| Policy ID | `attorney_review_required_before_external_reliance` |
| Artifact ID | `sha256:example-artifact-digest` |
| Receipt ID | `sha256:example-axle-receipt-digest` |
| Verification Status | `pass` |

## Defensibility Note

This bundle records matter-bound workflow evidence. It does not assert legal correctness, replace attorney judgment, determine privilege, or guarantee admissibility.
