namespace Lumio.Gen.ProtocolPermissionValidator
{
    // The executable ADR-022 Protocol/Permission gate.
    public enum Verdict { Accept, Reject }

    public readonly struct GateInput
    {
        public GateInput(string sessionId, string productId, string gameReleaseId, string messageId, string role, string[] claims, ulong connectionGeneration, string admittedSessionId, string admittedProductId, string admittedGameReleaseId, string admittedRole, string[] admittedClaims, ulong admittedConnectionGeneration)
        {
            SessionId = sessionId; ProductId = productId; GameReleaseId = gameReleaseId; MessageId = messageId; Role = role; Claims = claims; ConnectionGeneration = connectionGeneration; AdmittedSessionId = admittedSessionId; AdmittedProductId = admittedProductId; AdmittedGameReleaseId = admittedGameReleaseId; AdmittedRole = admittedRole; AdmittedClaims = admittedClaims; AdmittedConnectionGeneration = admittedConnectionGeneration;
        }
        public string SessionId { get; }
        public string ProductId { get; }
        public string GameReleaseId { get; }
        public string MessageId { get; }
        public string Role { get; }
        public string[] Claims { get; }
        public ulong ConnectionGeneration { get; }
        public string AdmittedSessionId { get; }
        public string AdmittedProductId { get; }
        public string AdmittedGameReleaseId { get; }
        public string AdmittedRole { get; }
        public string[] AdmittedClaims { get; }
        public ulong AdmittedConnectionGeneration { get; }
    }

    public static class ProtocolGate
    {
        public static readonly string[] RegisteredMessageIds =
        {
            "Handshake",
            "FullSnapshot",
            "Delta",
            "ResyncRequest",
            "MaintenanceKick",
            "BaselineAck",
            "DeltaAck",
            "Error",
        };

        /// <summary>Rejection precedence when more than one check fails (ADR-048).</summary>
        public static readonly string[] RejectPrecedence =
        {
            "StaleConnectionGeneration",
            "SessionMismatch",
            "ReleaseMismatch",
            "MessagePermissionDenied",
            "RoleMismatch",
            "ClaimNotGranted",
        };

        /// <summary>Reasons the session owner declares and the gate never derives.</summary>
        public static readonly string[] DeclaredOnlyReasons = { "SessionAntiReplay" };

        /// <summary>Runs the gate. A null reason means Accept. The messageId clause
        /// is enforced only as far as the architecture source publishes it: the id
        /// must be registered. No role-to-message table exists, so none is invented.</summary>
        public static Verdict Evaluate(GateInput input, out string? rejectReason)
        {
            if (input.ConnectionGeneration != input.AdmittedConnectionGeneration)
            { rejectReason = "StaleConnectionGeneration"; return Verdict.Reject; }
            if (input.SessionId != input.AdmittedSessionId)
            { rejectReason = "SessionMismatch"; return Verdict.Reject; }
            if (input.ProductId != input.AdmittedProductId || input.GameReleaseId != input.AdmittedGameReleaseId)
            { rejectReason = "ReleaseMismatch"; return Verdict.Reject; }
            if (System.Array.IndexOf(RegisteredMessageIds, input.MessageId) < 0)
            { rejectReason = "MessagePermissionDenied"; return Verdict.Reject; }
            if (input.Role != input.AdmittedRole)
            { rejectReason = "RoleMismatch"; return Verdict.Reject; }
            foreach (var claim in input.Claims)
            {
                if (System.Array.IndexOf(input.AdmittedClaims, claim) < 0)
                { rejectReason = "ClaimNotGranted"; return Verdict.Reject; }
            }
            rejectReason = null;
            return Verdict.Accept;
        }
    }
}
