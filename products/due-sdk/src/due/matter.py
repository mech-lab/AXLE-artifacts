"""Matter context binding for DUE receipts."""

from __future__ import annotations

from .models import MatterContext


def bind(
    *,
    matter_id: str,
    client: str,
    matter_type: str,
    jurisdiction: str,
    confidentiality: str,
) -> MatterContext:
    """Bind a DUE workflow to a legal matter context."""
    return MatterContext(
        schema="due.matter.v1",
        matter_id=matter_id,
        client=client,
        matter_type=matter_type,
        jurisdiction=jurisdiction,
        confidentiality=confidentiality,
    )
