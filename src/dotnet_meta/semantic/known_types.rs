//! Unchanged known-external vocabulary; split for the active-file ceiling.

pub(crate) fn known_csharp_type_identities() -> Vec<crate::ir::csharp_resolution::CSharpTypeIdentity>
{
    vec![
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(EF_CORE_DB_CONTEXT),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(AUTOMAPPER_PROFILE),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(SIGNALR_HUB),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external_with_arity(SIGNALR_HUB, 1),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(ASPNET_CONTROLLER),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(ASPNET_CONTROLLER_BASE),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(
            ASPNET_API_CONTROLLER_ATTRIBUTE,
        ),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(
            ASPNET_CONTROLLER_ATTRIBUTE,
        ),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(ASPNET_ROUTE_ATTRIBUTE),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(
            MSTEST_TEST_CLASS_ATTRIBUTE,
        ),
    ]
    .into_iter()
    .chain(ASPNET_HTTP_ATTRIBUTES.iter().map(|identity| {
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(*identity)
    }))
    .collect()
}

pub(super) const EF_CORE_DB_CONTEXT: &str = "Microsoft.EntityFrameworkCore.DbContext";
pub(super) const AUTOMAPPER_PROFILE: &str = "AutoMapper.Profile";
pub(super) const SIGNALR_HUB: &str = "Microsoft.AspNetCore.SignalR.Hub";
pub(super) const ASPNET_CONTROLLER: &str = "Microsoft.AspNetCore.Mvc.Controller";
pub(super) const ASPNET_CONTROLLER_BASE: &str = "Microsoft.AspNetCore.Mvc.ControllerBase";
pub(super) const ASPNET_API_CONTROLLER_ATTRIBUTE: &str =
    "Microsoft.AspNetCore.Mvc.ApiControllerAttribute";
pub(super) const ASPNET_CONTROLLER_ATTRIBUTE: &str = "Microsoft.AspNetCore.Mvc.ControllerAttribute";
pub(super) const ASPNET_ROUTE_ATTRIBUTE: &str = "Microsoft.AspNetCore.Mvc.RouteAttribute";
pub(super) const ASPNET_HTTP_ATTRIBUTES: &[&str] = &[
    "Microsoft.AspNetCore.Mvc.HttpGetAttribute",
    "Microsoft.AspNetCore.Mvc.HttpPostAttribute",
    "Microsoft.AspNetCore.Mvc.HttpPutAttribute",
    "Microsoft.AspNetCore.Mvc.HttpDeleteAttribute",
    "Microsoft.AspNetCore.Mvc.HttpPatchAttribute",
    "Microsoft.AspNetCore.Mvc.HttpHeadAttribute",
    "Microsoft.AspNetCore.Mvc.HttpOptionsAttribute",
];
pub(super) const MSTEST_TEST_CLASS_ATTRIBUTE: &str =
    "Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute";
