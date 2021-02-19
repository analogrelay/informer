using System;
using System.Buffers.Binary;
using System.Text;

namespace Informer.MySQL
{
    internal static class SpanExtensions
    {
        public static byte ReadByte(this ref ReadOnlySpan<byte> self)
        {
            var ret = self[0];
            self = self[1..];
            return ret;
        }

        public static ReadOnlySpan<byte> ReadBytes(this ref ReadOnlySpan<byte> self, int count)
        {
            if (count > self.Length)
            {
                throw new FormatException($"Unexpected end-of-buffer when reading {count} bytes.");
            }
            var ret = self[0..count];
            if (count < self.Length)
            {
                self = self[count..];
            }
            else
            {
                self = ReadOnlySpan<byte>.Empty;
            }
            return ret;
        }

        public static ushort ReadUInt16(this ref ReadOnlySpan<byte> self) =>
            BinaryPrimitives.ReadUInt16LittleEndian(self.ReadBytes(2));

        public static uint ReadUInt32(this ref ReadOnlySpan<byte> self) =>
            BinaryPrimitives.ReadUInt32LittleEndian(self.ReadBytes(4));

        public static string ReadNulTerminatedString(this ref ReadOnlySpan<byte> self, bool allowUnterminated = false)
        {
            var end = self.IndexOf((byte)0);
            if (end == -1)
            {
                if (!allowUnterminated)
                {
                    throw new FormatException("Reached the end of the buffer without finding a nul-terminator");
                }
                else
                {
                    end = self.Length;
                }
            }
            var str = self[..end];

            if (end < self.Length)
            {
                self = self[(end + 1)..];
            }
            else
            {
                self = ReadOnlySpan<byte>.Empty;
            }
            return Encoding.ASCII.GetString(str);
        }
    }
}
