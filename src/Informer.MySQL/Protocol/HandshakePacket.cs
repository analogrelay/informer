using System;

namespace Informer.MySQL.Protocol
{
    public record HandshakePacket
    {
        public byte ProtocolVersion { get; init; }
        public string ServerVersion { get; init; }
        public uint ConnectionId { get; init; }
        public ReadOnlyMemory<byte> AuthPluginData { get; init; }
        public CapabilityFlags Capabilities { get; init; }
        public byte DefaultCharacterSet { get; init; }
        public StatusFlags Status { get; init; }
        public string AuthPluginName { get; init; }

        public static HandshakePacket Read(ReadOnlySpan<byte> packet)
        {
            var protocolVersion = packet.ReadByte();
            if (protocolVersion != 0x0A)
            {
                throw new FormatException($"Unsupported protocol version: {protocolVersion}");
            }
            var serverVersion = packet.ReadNulTerminatedString();
            var connectionId = packet.ReadUInt32();
            var authPluginData = packet.ReadBytes(8);
            packet = packet[1..]; // Skip the filler
            var capabilityFlags = (CapabilityFlags)packet.ReadUInt16();

            byte characterSet = 0;
            var statusFlags = StatusFlags.NONE;
            var authPluginName = string.Empty;
            if (packet.Length > 0)
            {
                characterSet = packet.ReadByte();
                statusFlags = (StatusFlags)packet.ReadUInt16();
                capabilityFlags |= (CapabilityFlags)(((uint)packet.ReadUInt16()) << 16);

                var authDataLen = packet.ReadByte();
                packet = packet[10..]; // Skip the reserved section
                if (capabilityFlags.HasFlag(CapabilityFlags.CLIENT_SECURE_CONNECTION))
                {
                    var additionalDataLen = Math.Max(12, authDataLen - 9);
                    var additionalData = packet.ReadBytes(additionalDataLen);
                    packet = packet[1..];

                    var newData = new byte[authPluginData.Length + additionalData.Length];
                    authPluginData.CopyTo(newData);
                    additionalData.CopyTo(newData[authPluginData.Length..]);
                    authPluginData = newData;
                }

                authPluginName = packet.ReadNulTerminatedString(allowUnterminated: true);
            }

            return new HandshakePacket()
            {
                ProtocolVersion = protocolVersion,
                ServerVersion = serverVersion,
                ConnectionId = connectionId,
                AuthPluginData = authPluginData.ToArray(),
                Capabilities = capabilityFlags,
                DefaultCharacterSet = characterSet,
                Status = statusFlags,
                AuthPluginName = authPluginName
            };
        }
    }
}
