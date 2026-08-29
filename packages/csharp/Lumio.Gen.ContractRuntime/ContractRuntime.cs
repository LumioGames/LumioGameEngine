using System;
using System.Security.Cryptography;
using System.Text;

namespace Lumio.Gen.ContractRuntime
{
    public enum ChainBreak { Truncated, Mismatch }

    public static class HashChain
    {
        public static byte[] Append(byte[] prev, byte[] payload)
        {
            var buf = new byte[prev.Length + payload.Length];
            Buffer.BlockCopy(prev, 0, buf, 0, prev.Length);
            Buffer.BlockCopy(payload, 0, buf, prev.Length, payload.Length);
            return Sha256(buf);
        }
        public static bool Verify(byte[] prev, byte[] payload, byte[] expected)
        {
            var got = Append(prev, payload);
            if (got.Length != expected.Length) return false;
            for (var i = 0; i < got.Length; i++) { if (got[i] != expected[i]) return false; }
            return true;
        }
        public static byte[] Sha256(byte[] data)
        {
            using (var sha = SHA256.Create()) { return sha.ComputeHash(data); }
        }
    }

    public sealed class BoundedBuffer
    {
        private readonly byte[] _data; private int _len;
        public BoundedBuffer(int cap) { _data = new byte[cap]; }
        public bool TryPush(byte b) { if (_len >= _data.Length) return false; _data[_len++] = b; return true; }
        public int Length => _len;
    }

    public static class SelfTest
    {
        public static void HashChainRoundTrip()
        {
            var genesis = HashChain.Sha256(Array.Empty<byte>());
            var next = HashChain.Append(genesis, System.Text.Encoding.UTF8.GetBytes("rec-1"));
            if (!HashChain.Verify(genesis, System.Text.Encoding.UTF8.GetBytes("rec-1"), next))
                throw new InvalidOperationException("hash chain round-trip failed");
            if (HashChain.Verify(genesis, System.Text.Encoding.UTF8.GetBytes("rec-2"), next))
                throw new InvalidOperationException("hash chain must reject a mutated payload");
        }
        public static void TruncatedBuffer()
        {
            var buf = new BoundedBuffer(2);
            if (!buf.TryPush(1) || !buf.TryPush(2) || buf.TryPush(3))
                throw new InvalidOperationException("bounded buffer did not truncate");
        }
    }
}
