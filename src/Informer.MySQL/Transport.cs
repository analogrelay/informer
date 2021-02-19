using System;
using System.IO.Pipelines;
using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;

namespace Informer.MySQL
{
    public abstract class Transport : IAsyncDisposable
    {
        public abstract ValueTask<IDuplexPipe> ConnectAsync(CancellationToken cancellationToken = default);
        public virtual ValueTask DisposeAsync()
        {
            return ValueTask.CompletedTask;
        }
    }

    public class SocketTransport : Transport
    {
        private readonly Socket socket;
        private readonly EndPoint endPoint;
        private Task recieving;
        private Task sending;
        private readonly CancellationTokenSource stopping = new();
        private Pipe socketReceiving;
        private Pipe socketSending;

        public SocketTransport(EndPoint endPoint)
        {
            this.socket = new Socket(endPoint.AddressFamily, SocketType.Stream, ProtocolType.Tcp);
            this.endPoint = endPoint;
        }

        public override async ValueTask<IDuplexPipe> ConnectAsync(CancellationToken cancellationToken = default)
        {
            await socket.ConnectAsync(endPoint);

            socketReceiving = new Pipe();
            socketSending = new Pipe();
            recieving = ReceiveLoopAsync(socketReceiving.Writer, socket, stopping.Token);
            sending = SendLoopAsync(socketSending.Reader, socket, stopping.Token);

            return new DuplexPipe(socketReceiving.Reader, socketSending.Writer);
        }

        private static async Task SendLoopAsync(PipeReader reader, Socket socket, CancellationToken cancellationToken)
        {
            while (!cancellationToken.IsCancellationRequested)
            {
                var result = await reader.ReadAsync();
                if (result.IsCanceled)
                {
                    return;
                }
                var buffer = result.Buffer;

                foreach (var segment in buffer)
                {
                    await socket.SendAsync(segment, SocketFlags.None, cancellationToken);
                }

                if (result.IsCompleted)
                {
                    return;
                }
            }
        }

        private static async Task ReceiveLoopAsync(PipeWriter writer, Socket socket, CancellationToken cancellationToken)
        {
            const int minimumBufferSize = 512;
            while (!cancellationToken.IsCancellationRequested)
            {
                var buffer = writer.GetMemory(minimumBufferSize);
                var bytesRead = await socket.ReceiveAsync(buffer, SocketFlags.None, cancellationToken);
                if (bytesRead == 0)
                {
                    return;
                }
                writer.Advance(bytesRead);

                var result = await writer.FlushAsync();
                if (result.IsCanceled || result.IsCompleted)
                {
                    return;
                }
            }
        }

        public override async ValueTask DisposeAsync()
        {
            stopping.Cancel();
            socket.Dispose();
            socketSending?.Writer.Complete();
            socketSending?.Reader.CancelPendingRead();
            socketReceiving?.Writer.CancelPendingFlush();
            socketReceiving?.Reader.Complete();
            await Task.WhenAll(sending, recieving);
        }
    }
}
