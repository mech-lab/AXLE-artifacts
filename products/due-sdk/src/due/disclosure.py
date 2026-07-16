"""Disclosure posture records for DUE duty actions."""

from __future__ import annotations

from .models import DisclosureRecord, DutyAction, make_id


def record(
    *,
    action: DutyAction,
    status: str,
    audience: str,
    note: str | None = None,
) -> DisclosureRecord:
    """Record whether, how, and to whom an AI-assisted output was disclosed."""
    return DisclosureRecord(
        schema="due.disclosure.v1",
        disclosure_id=make_id("disc"),
        action_id=action.action_id,
        status=status,
        audience=audience,
        note=note,
    )
