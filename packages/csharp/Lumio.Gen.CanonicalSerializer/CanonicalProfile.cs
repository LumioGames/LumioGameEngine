using System;

namespace Lumio.Gen.CanonicalSerializer;

public static class CanonicalForm
{
    public const string FormId = "CanonicalJsonV1";
    public const string Encoding = "AsciiEscaped";
    public const string MemberOrder = "CodePointAscending";
    public const string ArrayOrder = "DocumentOrder";
    public const char ItemSeparator = ',';
    public const char KeyValueSeparator = ':';
    public const string Numbers = "IntegerOnly";
    public const string UnknownMembers = "Reject";
    public const string DuplicateMembers = "Reject";
    public const string DigestAlgorithm = "SHA-256";
    public const string DigestFraming = "PrefixFreeOverCanonicalBytes";
}

public readonly record struct DigestDomain(string Digest, string DomainTag, string SortRule, string[] OmitMembers);
public static class DigestDomains
{
    public static readonly DigestDomain[] All =
    {
        new DigestDomain("manifestDigest", "CoreEngineManifestBody", "member order only; the body has no array whose order is semantic", System.Array.Empty<string>()),
        new DigestDomain("artifactSetDigest", "ArtifactSetV1", "entries sorted ascending by path (code point); paths are unique within an index", new[] { "artifactSetDigest" }),
        new DigestDomain("artifactIndexDigest", "ArtifactIndexV1", "index.entries sorted ascending by path (code point)", System.Array.Empty<string>()),
        new DigestDomain("targetProfileDigest", "TargetProfileV1", "member order only; the profile has no array", System.Array.Empty<string>()),
        new DigestDomain("capabilitySetDigest", "CapabilitySetV1", "capabilities sorted ascending by code point; the array is uniqueItems so ties are impossible", System.Array.Empty<string>()),
    };
}

public readonly record struct CanonicalGolden(string Id, string Case, string Sha256);
public static class CanonicalGoldens
{
    public static readonly CanonicalGolden[] All =
    {
        new CanonicalGolden("artifact-set-empty", "EmptyArtifactSet", "7a92ee35f0ae0644282f438a675d7624800a8aeac5125c85d7796d844831ce69"),
        new CanonicalGolden("artifact-set-single", "SingleArtifact", "b5102a58a8338bff3200949d01c341abaa66527628ea91cdaa7a928987e9a7e9"),
        new CanonicalGolden("artifact-set-multi", "MultiArtifact", "7d851d441b751469da4c5e935736fe7684b18db9387d195a3dcbd48684d9a365"),
        new CanonicalGolden("artifact-set-path-permutation", "PathOrderPermutation", "7d851d441b751469da4c5e935736fe7684b18db9387d195a3dcbd48684d9a365"),
        new CanonicalGolden("capability-set-permutation", "CapabilityOrderPermutation", "6c0859b0a2c6624a7725bcbdb2d71b1e2a65bbf413cac6874875dc69fc54d19a"),
        new CanonicalGolden("escape-boundary", "EscapeBoundary", "b7fb9362f56ca50b68969d61dd73ee73f31a23a64d4f15a898045c1f6c4cb5eb"),
        new CanonicalGolden("integer-boundary", "IntegerBoundary", "a6135f19ccbcd596fb0bd7bc81fcd9770f49609c483812fc776bf71c28c2de73"),
        new CanonicalGolden("artifact-set-schema-version", "SchemaVersionChange", "e0fd1f58d4435bec2365841d68280a6bf03aa158e99de07c3e970f7bc836968e"),
    };
}
