using System;
using System.Text;
using Xunit;

namespace Informer.MySQL.Tests
{
    public class SpanExtensionsTests
    {
        [Fact]
        public void ReadByte()
        {
            ReadOnlySpan<byte> input = new byte[] { 0, 1, 2, 3 }.AsSpan();
            Assert.Equal(0, input.ReadByte());
            Assert.Equal(1, input.ReadByte());
            Assert.Equal(2, input.ReadByte());
            Assert.Equal(3, input.ReadByte());
        }

        [Fact]
        public void ReadNulTerminatedString()
        {
            var str = "hello";
            var buf = new byte[str.Length + 1];
            Encoding.ASCII.GetBytes(str.AsSpan(), buf.AsSpan());
            Assert.Equal(0, buf[^1]);

            ReadOnlySpan<byte> input = buf;
            Assert.Equal("hello", input.ReadNulTerminatedString());
        }

        [Fact]
        public void ReadNulTerminatedString_ThrowsWhenNoTerminatorByDefault()
        {
            var str = "hello";
            var buf = new byte[str.Length];
            Encoding.ASCII.GetBytes(str.AsSpan(), buf.AsSpan());

            Assert.Throws<FormatException>(() =>
            {
                ReadOnlySpan<byte> input = buf;
                input.ReadNulTerminatedString();
            });
        }

        [Fact]
        public void ReadNulTerminatedString_AcceptsStringToEndOfBufferWhenSpecified()
        {
            var str = "hello";
            var buf = new byte[str.Length];
            Encoding.ASCII.GetBytes(str.AsSpan(), buf.AsSpan());

            ReadOnlySpan<byte> input = buf;
            Assert.Equal("hello", input.ReadNulTerminatedString(allowUnterminated: true));
        }

        [Fact]
        public void ReadUInt32()
        {
            ReadOnlySpan<byte> buf = new byte[] { 0xAB, 0xCD, 0xEF, 0x12 };
            Assert.Equal(0x12EFCDABu, buf.ReadUInt32());
        }

        [Fact]
        public void ReadUInt16()
        {
            ReadOnlySpan<byte> buf = new byte[] { 0xAB, 0xCD };
            Assert.Equal(0xCDABu, buf.ReadUInt16());
        }

        [Fact]
        public void ReadTooManyBytes()
        {
            Assert.Throws<FormatException>(() =>
            {
                ReadOnlySpan<byte> buf = new byte[] { 0xAB, 0xCD };
                buf.ReadBytes(4);
            });
        }
    }
}
