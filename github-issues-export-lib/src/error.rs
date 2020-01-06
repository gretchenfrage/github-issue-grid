
error_chain!{
    errors {
        Request(t: String) {
            description("invalid request name")
            display("request failed: '{}'", t)
        }
    }
    foreign_links {
        Docopt(::docopt::Error);
        Io(::std::io::Error);
        Hyper(::hyper::Error);
        HbTemplate(::handlebars::TemplateError);
        HbRender(::handlebars::RenderError);
        NativeTls(::native_tls::Error);
        Json(::serde_json::Error);
        Utf8(::std::str::Utf8Error);
        EnvVar(::std::env::VarError);
    }
}