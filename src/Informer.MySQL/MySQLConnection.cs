using System;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using System.Threading.Tasks;
using Informer.MySQL.Protocol;

namespace Informer.MySQL
{
    public class MySQLConnection
    {
        private readonly Transport transport;
        private Connection connection;

        public MySQLConnection(string host, int port)
            : this(new SocketTransport(new DnsEndPoint(host, port, AddressFamily.InterNetwork)))
        {
        }

        public MySQLConnection(Transport transport)
        {
            this.transport = transport;
        }

        public async Task ConnectAsync(CancellationToken cancellationToken = default)
        {
            var pipe = await transport.ConnectAsync(cancellationToken);
            connection = new Connection(pipe);

            var packet = await connection.ReadPacketAsync(cancellationToken);
            Console.WriteLine($"Read {packet.Payload.Length} byte packet (Seq ID: {packet.SequenceId})");

            var handshake = HandshakePacket.Read(packet.Payload.Span);
            Console.WriteLine($"Handshake: {handshake}");
        }
    }
}
