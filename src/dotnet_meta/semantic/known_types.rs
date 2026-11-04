//! Unchanged known-external vocabulary; split for the active-file ceiling.
use super::*;

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
