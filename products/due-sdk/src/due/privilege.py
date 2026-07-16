"""Privilege and confidentiality records for DUE duty actions."""

from __future__ import annotations

from .models import DutyAction, PrivilegeRecord, make_id


def record(
    *,
    action: DutyAction,
    status: str,
    basis: str,
    reviewer: str | None = None,
) -> PrivilegeRecord:
    """Record the privilege or confidentiality posture for an AI-assisted action."""
    return PrivilegeRecord(
        schema="due.privilege.v1",
        privilege_id=make_id("priv"),
        action_id=action.action_id,
        status=status,
        basis=basis,
        reviewer=reviewer,
    )
