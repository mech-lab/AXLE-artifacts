"""AXLE receipt references for proof-backed DUE records."""

from __future__ import annotations

from typing import Literal

from .models import AxleReceiptReference, DutyAction, make_id


def attach_axle_receipt(
    *,
    action: DutyAction,
    artifact_id: str,
    receipt_id: str,
    policy_id: str,
    verification_status: Literal["pass", "fail", "unknown"] = "unknown",
    note: str | None = None,
) -> AxleReceiptReference:
    """Attach an AXLE receipt reference to a DUE duty action.

    This does not re-verify the `.axle` artifact. It records a reference to a
    proof artifact or AXLE receipt that can be checked by AXLE-rs tooling.
    """
    return AxleReceiptReference(
        schema="due.axle_receipt_reference.v1",
        proof_id=make_id("proof"),
        action_id=action.action_id,
        artifact_id=artifact_id,
        receipt_id=receipt_id,
        policy_id=policy_id,
        verification_status=verification_status,
        note=note,
    )
