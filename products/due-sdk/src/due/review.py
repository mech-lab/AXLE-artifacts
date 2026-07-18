"""Human review records for DUE duty actions."""

from __future__ import annotations

from .models import DutyAction, ReviewRecord, make_id


def record(
    *,
    action: DutyAction,
    reviewer: str,
    role: str,
    decision: str,
    note: str | None = None,
) -> ReviewRecord:
    """Record qualified human review for an AI-assisted legal action."""
    return ReviewRecord(
        schema="due.review.v1",
        review_id=make_id("rev"),
        action_id=action.action_id,
        reviewer=reviewer,
        role=role,
        decision=decision,
        note=note,
    )
