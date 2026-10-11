public interface IRunnable { void Run(); }
public class Alpha : IRunnable {
  public void Run() {
    service.Save<User>(user: dto, count: 2);
    Lookup(dto);
    Lookup(dto);
    Factory.Create().Load(First());
    handlers[name](dto);
    // Phantom(dto);
    var text = "Phantom(dto)";
  }
  public void Run(int value) { Numeric(value); }
}
public class Beta { public void Run() { OnlyBeta(); } }
