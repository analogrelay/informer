using System.Threading.Tasks;
using Informer.MySQL;
using McMaster.Extensions.CommandLineUtils;

namespace informer
{
    [Command("trace", Description = "Trace binlog events from the specified server.")]
    internal class TraceCommand
    {
        [Option("-h|--host <HOST>", Description = "The host to connect to. Defaults to 'localhost'")]
        public string Host { get; set; } = "localhost";

        [Option("-P|--port <PORT>", Description = "The port to connect to. Defaults to 3306.")]
        public int Port { get; set; } = 3306;

        [Option("-u|--user <USER>", Description = "The user to connect with. Defaults to 'root'.")]
        public string User { get; set; } = "root";

        [Option("-p|--password", Description = "Set this flag to prompt for a password.")]
        public bool PromptPassword { get; set; }

        public async Task<int> OnExecuteAsync(CommandLineApplication app, IConsole console)
        {
            var connection = new MySQLConnection(Host, Port);
            await connection.ConnectAsync();

            return 0;
        }
    }
}
