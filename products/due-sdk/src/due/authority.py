"""Authority records for DUE duty actions."""

from __future__ import annotations

from .models import AuthorityRecord, DutyAction, make_id


def record(
    *,
    action: DutyAction,
    authority_type: str,
    source: str,
    rule: str,
) -> AuthorityRecord:
    """Record the policy, engagement scope, rule, or instruction authorizing an action."""
    return AuthorityRecord(
        schema="due.authority.v1",
        authority_id=make_id("auth"),
        action_id=action.action_id,
        authority_type=authority_type,
        source=source,
        rule=rule,
    )
