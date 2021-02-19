using System;
using McMaster.Extensions.CommandLineUtils;

namespace informer
{
    [Command("informer", Description = "Watching your DB")]
    [Subcommand(typeof(TraceCommand))]
    class Program
    {
        static void Main(string[] args) => CommandLineApplication.Execute<Program>(args);

        private int OnExecute(CommandLineApplication app)
        {
            app.ShowHelp();
            return 0;
        }
    }
}
