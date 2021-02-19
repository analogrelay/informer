using System;

namespace Informer.MySQL.Protocol
{
    public readonly struct Packet
    {
        public static readonly Packet Empty = new(0, ReadOnlyMemory<byte>.Empty);

        public int SequenceId { get; }
        public ReadOnlyMemory<byte> Payload { get; }

        public Packet(int sequenceId, ReadOnlyMemory<byte> payload)
        {
            SequenceId = sequenceId;
            Payload = payload;
        }
    }
}
