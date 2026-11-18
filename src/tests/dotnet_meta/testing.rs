use crate::compression::Fidelity;
use crate::config::{CleanCtxConfig, MetaLayerConfig};
use crate::dotnet_meta::{DotNetMetaLayer, run_meta_layer};
use crate::ir::pipeline::{PassContext, PassPipeline};
use crate::layers::meta::MetaLayer;
use crate::layers::meta::semantic::SemanticRelation;
use crate::workspace::index::WorkspaceIndex;

const TEST_CLASS: &str = r#"
[TestClass]
public class AccountsControllerTests
{
    private readonly Mock<IAccountService> _accounts;
    private Mock<IRepository> _repository;
    private AccountsController _controller;

    [TestInitialize]
    public void Setup()
    {
        _controller = new AccountsController(_accounts.Object);
        var request = new AccountRequest();
        _accounts.Setup(x => x.GetAccount(1)).ReturnsAsync(account);
        _repository
            .Setup(x => x.Save(It.IsAny<int>()))
            .Returns(result);
    }

    [TestMethod]
    public async Task Returns_account() { }

    [DataTestMethod]
    [DataRow(1)]
    [DataRow(2)]
    public void Maps_status(int status) { }

    [ClassInitialize]
    public static void BeforeAll(TestContext context) { }

    [TestCleanup]
    public void Cleanup() { }

    public void VerifyCalls()
    {
        _accounts.Verify(x => x.Save(), Times.Once());
        _accounts.Verify(x => x.Delete(), Times.Never());
        _accounts.Verify(
            x => x.Update(It.IsAny<int>()),
            Times.Exactly(3));
        _accounts.Verify(x => x.Dynamic(), Times.Exactly(expected));
    }
}
"#;

fn rendered(fidelity: Fidelity) -> String {
    run_meta_layer(TEST_CLASS, &[TEST_CLASS.to_string()], fidelity)
        .expect("testing metadata")
        .render()
}

#[test]
fn red_t2_low_emits_only_test_class() {
    let output = rendered(Fidelity::Low);
    assert!(output.contains("Φtestcls:AccountsControllerTests"));
    for absent in ["Φtest:", "Φmock:", "Φsetup:", "Φverify:", "Φfixture:"] {
        assert!(!output.contains(absent), "unexpected {absent} in {output}");
    }
}

#[test]
fn red_t3_t4_medium_emits_methods_rows_and_deduplicated_mocks() {
    let output = rendered(Fidelity::Medium);
    assert!(output.contains("Φtest:Returns_account"));
    assert!(output.contains("Φtest:Maps_status [rows=2]"));
    assert_eq!(output.matches("Φmock:IAccountService").count(), 1);
    assert_eq!(output.matches("Φmock:IRepository").count(), 1);
    for absent in ["Φsetup:", "Φverify:", "Φfixture:"] {
        assert!(!output.contains(absent));
    }
}

#[test]
fn red_t5_t6_t7_high_emits_moq_behavior_and_fixtures() {
    let output = rendered(Fidelity::High);
    assert!(output.contains("Φsetup:GetAccount → account"));
    assert!(output.contains("Φsetup:Save → result"));
    assert!(output.contains("Φverify:Save [times=1]"));
    assert!(output.contains("Φverify:Delete [times=0]"));
    assert!(output.contains("Φverify:Update [times=3]"));
    assert!(output.contains("Φverify:Dynamic\n"));
    assert!(output.contains("Φfixture:Setup"));
    assert!(output.contains("Φfixture:BeforeAll"));
    assert!(output.contains("Φfixture:Cleanup"));
}

#[test]
fn red_t8_configuration_disables_only_testing_metadata_and_edges() {
    let source = format!(
        "{TEST_CLASS}\n[ApiController]\npublic class HealthController : ControllerBase {{}}"
    );
    let captures = vec![
        TEST_CLASS.to_string(),
        "[ApiController]\npublic class HealthController : ControllerBase {}".to_string(),
    ];
    let mut config = CleanCtxConfig::default();
    let mut dotnet = MetaLayerConfig::default();
    dotnet.testing.enabled = false;
    config.meta_layers.insert("dotnet".to_string(), dotnet);
    let layer = DotNetMetaLayer::new();

    let output = layer
        .enrich(&source, &captures, Fidelity::High, Some(&config))
        .expect("unrelated ASP.NET metadata remains")
        .rendered;
    assert!(output.contains("Φctrl:HealthController"));
    assert!(!output.contains("Φtestcls:"));
    assert!(
        layer
            .extract_semantic_edges(&source, &captures, Fidelity::High, Some(&config))
            .iter()
            .all(|edge| {
                edge.relation != SemanticRelation::Tests
                    && !(edge.relation == SemanticRelation::Defines
                        && edge.subject.domain == "dotnet"
                        && edge.subject.entity_type == "TestClass")
            })
    );
}

#[test]
fn candidate_precedence_and_ambiguity_are_source_local() {
    let layer = DotNetMetaLayer::new();
    let named =
        layer.extract_semantic_edges(TEST_CLASS, &[TEST_CLASS.to_string()], Fidelity::Low, None);
    let declaration = named
        .iter()
        .find(|edge| edge.relation == SemanticRelation::Defines)
        .expect("TestClass declaration");
    assert_eq!(declaration.subject.name, "AccountsControllerTests");
    let candidate = crate::dotnet_meta::testing::extract_testing_subject_candidate(TEST_CLASS)
        .expect("candidate");
    assert_eq!(candidate.subject, "AccountsController");

    let ambiguous = r#"
[TestClass]
public class BehaviorChecks
{
    private FooService _foo;
    private BarService _bar;
    [TestInitialize]
    public void Setup()
    {
        _foo = new FooService();
        _bar = new BarService();
    }
}
"#;
    assert!(crate::dotnet_meta::testing::extract_testing_subject_candidate(ambiguous).is_none());

    let excluded_suffix = "[TestClass]\npublic class PaymentIntegrationTests {}";
    assert!(
        crate::dotnet_meta::testing::extract_testing_subject_candidate(excluded_suffix).is_none()
    );
}

#[test]
fn persistent_constructed_subject_is_candidate_evidence() {
    let source = r#"
[TestClass]
public class ServiceBehavior
{
    private FooService _sut;
    [TestInitialize]
    public void Setup() { _sut = new FooService(); }
}
"#;
    let candidate = crate::dotnet_meta::testing::extract_testing_subject_candidate(source)
        .expect("constructed candidate");
    assert_eq!(candidate.test_class, "ServiceBehavior");
    assert_eq!(candidate.subject, "FooService");
}

#[test]
fn production_pipeline_publishes_testclass_declaration_without_tests_edge() {
    let source = r#"
public class AccountsController { }

[TestClass]
public class AccountsControllerTests
{
    [TestMethod]
    public void Works() { }
}
"#;
    let mut context = PassContext::new(
        source.to_string(),
        "AccountsControllerTests.cs".to_string(),
        Fidelity::High,
    );
    context.canonical_path = Some("C:/repo/AccountsControllerTests.cs".to_string());
    context.language =
        Some(crate::compression::language::safe_csharp_language().expect("C# grammar"));
    context.query_string = crate::queries::CS_QUERY.to_string();
    PassPipeline::default_production()
        .run(&mut context)
        .expect("production pipeline");

    let mut index = WorkspaceIndex::new();
    index.add_edges("C:/repo/AccountsControllerTests.cs", context.semantic_edges);
    let declarations =
        index.declarations_by_identity("dotnet", "TestClass", "AccountsControllerTests");
    assert_eq!(declarations.len(), 1);
    assert!(
        index
            .forward_edges_by_identity("dotnet", "TestClass", "AccountsControllerTests")
            .is_empty(),
        "candidate evidence must not become a durable Tests edge"
    );
}
