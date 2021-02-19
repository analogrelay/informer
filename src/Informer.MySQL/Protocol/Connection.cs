using System;
using System.Buffers;
using System.IO.Pipelines;
using System.Threading;
using System.Threading.Tasks;

namespace Informer.MySQL.Protocol
{
    internal class Connection
    {
        private readonly IDuplexPipe pipe;

        public Connection(IDuplexPipe pipe)
        {
            this.pipe = pipe;
        }

        public async ValueTask WritePacketAsync(Packet packet, CancellationToken cancellationToken = default)
        {
            var payload = packet.Payload;
            while (payload.Length > 0)
            {
                var buf = pipe.Output.GetMemory(payload.Length);
                var toWrite = Math.Min(payload.Length, buf.Length);
                payload[0..toWrite].CopyTo(buf);
                payload = payload[toWrite..];
                await pipe.Output.FlushAsync(cancellationToken);
            }
        }

        public async ValueTask<Packet> ReadPacketAsync(CancellationToken cancellationToken = default)
        {
            var result = await pipe.Input.ReadAsync(cancellationToken);
            if (result.IsCanceled)
            {
                throw new OperationCanceledException();
            }

            var buffer = result.Buffer;
            while (buffer.Length < 4)
            {
                pipe.Input.AdvanceTo(buffer.Start, buffer.End);
                result = await pipe.Input.ReadAsync(cancellationToken);
                if (result.IsCanceled)
                {
                    throw new OperationCanceledException();
                }
                else if (result.IsCompleted && buffer.Length < 4)
                {
                    return Packet.Empty;
                }
                buffer = result.Buffer;
            }

            var header = buffer.Slice(0, 4);
            var (length, sequenceId) = ParseHeader(header);
            buffer = buffer.Slice(4);

            while (buffer.Length < length)
            {
                pipe.Input.AdvanceTo(buffer.Start, buffer.End);
                result = await pipe.Input.ReadAsync(cancellationToken);
                if (result.IsCanceled)
                {
                    throw new OperationCanceledException();
                }
                else if (result.IsCompleted && buffer.Length < length)
                {
                    return Packet.Empty;
                }
                buffer = result.Buffer;
            }

            var packet = new Packet(sequenceId, buffer.Slice(0, length).ToArray());
            buffer = buffer.Slice(length);
            pipe.Input.AdvanceTo(buffer.Start);
            return packet;
        }

        private (int length, int sequenceId) ParseHeader(ReadOnlySequence<byte> header)
        {
            static (int length, int sequenceId) ParseHeaderSingleSpan(ReadOnlySpan<byte> buf)
            {
                return (
                    buf[0] | (buf[1] << 8) | (buf[2] << 16),
                    buf[3]
                );
            }

            if (header.IsSingleSegment)
            {
                return ParseHeaderSingleSpan(header.FirstSpan);
            }
            else
            {
                Span<byte> buf = stackalloc byte[4];
                header.CopyTo(buf);
                return ParseHeaderSingleSpan(buf);
            }
        }
    }
}
