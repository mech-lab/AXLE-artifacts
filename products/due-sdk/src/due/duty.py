"""Duty action recording for AI-assisted legal workflows."""

from __future__ import annotations

from .models import DutyAction, MatterContext, make_id


def record_action(
    *,
    matter: MatterContext,
    action_type: str,
    actor: str,
    issue: str,
    output: str,
) -> DutyAction:
    """Record an AI-assisted action performed inside a duty-bound workflow."""
    return DutyAction(
        schema="due.duty_action.v1",
        action_id=make_id("act"),
        matter_id=matter.matter_id,
        action_type=action_type,
        actor=actor,
        issue=issue,
        output=output,
    )
