use ruff_python_ast::ExprCall;

use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_python_ast::name::QualifiedName;
use ruff_text_size::Ranged;

use crate::Violation;
use crate::checkers::ast::Checker;

/// ## What it does
/// Checks that async functions do not contain blocking HTTP calls.
///
/// ## Why is this bad?
/// Blocking an async function via a blocking HTTP call will block the entire
/// event loop, preventing it from executing other tasks while waiting for the
/// HTTP response, negating the benefits of asynchronous programming.
///
/// Instead of making a blocking HTTP call, use an asynchronous HTTP client
/// library such as `aiohttp` or `httpx`.
///
/// ## Example
/// ```python
/// import urllib
///
///
/// async def fetch():
///     urllib.request.urlopen("https://example.com/foo/bar").read()
/// ```
///
/// Use instead:
/// ```python
/// import aiohttp
///
///
/// async def fetch():
///     async with aiohttp.ClientSession() as session:
///         async with session.get("https://example.com/foo/bar") as resp:
///             ...
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct BlockingHttpCallInAsyncFunction;

impl Violation for BlockingHttpCallInAsyncFunction {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Async functions should not call blocking HTTP methods".to_string()
    }
}

fn is_blocking_http_call(qualified_name: &QualifiedName) -> bool {
    matches!(
        qualified_name.segments(),
        ["urllib", "request", "urlopen"]
            | ["urllib3", "request"]
            | [
                "httpx" | "requests",
                "get"
                    | "post"
                    | "delete"
                    | "patch"
                    | "put"
                    | "head"
                    | "connect"
                    | "options"
                    | "trace"
            ]
    )
}

/// ASYNC210
pub(crate) fn blocking_http_call(checker: &Checker, call: &ExprCall) {
    if checker.semantic().in_async_context() {
        if checker
            .semantic()
            .resolve_qualified_name(call.func.as_ref())
            .as_ref()
            .is_some_and(is_blocking_http_call)
        {
            checker.report_diagnostic(BlockingHttpCallInAsyncFunction, call.func.range());
        }
    }
}
