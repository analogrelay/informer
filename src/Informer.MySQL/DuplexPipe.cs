using System.IO.Pipelines;

namespace Informer.MySQL
{
    internal record DuplexPipe(PipeReader Input, PipeWriter Output) : IDuplexPipe
    {
    }
}
