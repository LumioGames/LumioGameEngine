//! Generated ProtocolPermissionValidator artifact. Do not hand-edit.
//! Publisher: LumioGameEngineArchitecture / LGE-V1.4-2026-08-27.

#![forbid(unsafe_code)]

pub const ACTIVE_PERMISSION_FIELDS: &[&str] = &[
    "sessionId",
    "productId",
    "gameReleaseId",
    "messageId",
    "role",
    "claims",
    "connectionGeneration",
    "antiReplay",
    "admittedSessionId",
    "admittedProductId",
    "admittedGameReleaseId",
    "admittedRole",
    "admittedClaims",
    "admittedConnectionGeneration",
    "verdict",
];

pub fn is_active_field(name: &str) -> bool {
    ACTIVE_PERMISSION_FIELDS.contains(&name)
}

/// Registered `MessageType` ids (ids/index.json, registry order).
pub const REGISTERED_MESSAGE_IDS: &[&str] = &[
    "Handshake",
    "FullSnapshot",
    "Delta",
    "ResyncRequest",
    "MaintenanceKick",
    "BaselineAck",
    "DeltaAck",
    "Error",
];

/// Rejection precedence when a record fails more than one check (ADR-048).
pub const REJECT_PRECEDENCE: &[&str] = &[
    "StaleConnectionGeneration",
    "SessionMismatch",
    "ReleaseMismatch",
    "MessagePermissionDenied",
    "RoleMismatch",
    "ClaimNotGranted",
];

/// Reasons the session owner declares and the gate never derives (ADR-022).
pub const DECLARED_ONLY_REASONS: &[&str] = &["SessionAntiReplay"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Accept,
    Reject,
}

/// One Active-session message and the context it was admitted under.
#[derive(Clone, Copy, Debug)]
pub struct GateInput<'a> {
    pub session_id: &'a str,
    pub product_id: &'a str,
    pub game_release_id: &'a str,
    pub message_id: &'a str,
    pub role: &'a str,
    pub claims: &'a [&'a str],
    pub connection_generation: u64,
    pub admitted_session_id: &'a str,
    pub admitted_product_id: &'a str,
    pub admitted_game_release_id: &'a str,
    pub admitted_role: &'a str,
    pub admitted_claims: &'a [&'a str],
    pub admitted_connection_generation: u64,
}

/// The ADR-022 gate. `None` reason means Accept.
///
/// The `messageId` clause is enforced only as far as this repository
/// publishes it -- the id must be registered. No role-to-message
/// permission table exists, so the gate does not invent one.
pub fn evaluate(input: &GateInput) -> (Verdict, Option<&'static str>) {
    if input.connection_generation != input.admitted_connection_generation {
        return (Verdict::Reject, Some("StaleConnectionGeneration"));
    }
    if input.session_id != input.admitted_session_id {
        return (Verdict::Reject, Some("SessionMismatch"));
    }
    if input.product_id != input.admitted_product_id
        || input.game_release_id != input.admitted_game_release_id
    {
        return (Verdict::Reject, Some("ReleaseMismatch"));
    }
    if !REGISTERED_MESSAGE_IDS.contains(&input.message_id) {
        return (Verdict::Reject, Some("MessagePermissionDenied"));
    }
    if input.role != input.admitted_role {
        return (Verdict::Reject, Some("RoleMismatch"));
    }
    let mut i = 0;
    while i < input.claims.len() {
        if !input.admitted_claims.contains(&input.claims[i]) {
            return (Verdict::Reject, Some("ClaimNotGranted"));
        }
        i += 1;
    }
    (Verdict::Accept, None)
}
