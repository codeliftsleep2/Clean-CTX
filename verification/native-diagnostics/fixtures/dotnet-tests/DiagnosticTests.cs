using Microsoft.VisualStudio.TestTools.UnitTesting;

[assembly: DoNotParallelize]

[TestClass]
public sealed class DiagnosticTests
{
    [TestMethod]
    public void InstalledRuntimeMatchesTarget() => Assert.AreEqual(10, Environment.Version.Major);

    [TestMethod]
    public void ControlledOutcome()
    {
#if PROBE_FAIL
        Assert.Fail("CTX_OPERATOR_FORCED_FAILURE");
#else
        Assert.AreEqual("DiagnosticTests", Path.GetFileNameWithoutExtension(typeof(DiagnosticTests).Assembly.Location));
#endif
    }
}
