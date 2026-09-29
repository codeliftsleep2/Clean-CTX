using System;

public interface IRunner : IBaseRunner
{
    string Run(string value);
}

public class Worker : BaseWorker, IRunner
{
    private readonly IRepository repository;

    public Worker(IRepository repository)
    {
        this.repository = repository;
    }

    public string Run(string value)
    {
        Console.WriteLine(value);
        return repository.Find(value);
    }
}
