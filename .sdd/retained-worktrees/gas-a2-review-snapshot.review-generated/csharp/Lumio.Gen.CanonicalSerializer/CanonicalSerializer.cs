namespace Lumio.Gen.CanonicalSerializer
{
    public static class SnapshotChecksum
    {
        public const string Domain = "SHA-256 over canonical JSON of snapshot-header minus checksum and hash fields";
        public const string Magic = "LUMIOSNP1";
    }
}
