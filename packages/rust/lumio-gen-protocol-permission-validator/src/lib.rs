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
