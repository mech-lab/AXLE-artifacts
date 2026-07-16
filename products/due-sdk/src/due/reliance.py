"""Reliance records for DUE duty actions."""

from __future__ import annotations

from .models import DutyAction, RelianceRecord, make_id


def record(
    *,
    action: DutyAction,
    relied_upon_by_human: bool,
    reliance_type: str,
    final_decision_maker: str | None = None,
) -> RelianceRecord:
    """Record whether and how a human relied on an AI-assisted output."""
    return RelianceRecord(
        schema="due.reliance.v1",
        reliance_id=make_id("rel"),
        action_id=action.action_id,
        relied_upon_by_human=relied_upon_by_human,
        reliance_type=reliance_type,
        final_decision_maker=final_decision_maker,
    )
