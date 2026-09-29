public interface IWorker
{
    string Run(string value);
}

public sealed class PeerWorker : IWorker
{
    public string Run(string value)
    {
        return value.Trim();
    }
}
