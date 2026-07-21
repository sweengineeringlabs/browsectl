/// The process's argument list, with the program name already stripped —
/// wrapped in a named type so `Client::dispatch`'s contract can gain fields
/// (e.g. an environment snapshot) later without a breaking signature change.
pub struct DispatchRequest {
    pub args: Vec<String>,
}
