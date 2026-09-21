import java.util.List;

public class Worker extends BaseWorker implements Runner {
    private final Repository repository;

    public Worker(Repository repository) {
        this.repository = repository;
    }

    public String run(String value) {
        audit(value);
        return repository.find(value);
    }
}

interface Runner extends BaseRunner {
    String run(String value);
}
