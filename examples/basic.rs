use fluent_setters::FluentSetters;

#[derive(Default, FluentSetters)]
struct Builder {
    a: i32,
    b: String,
}

fn main() {
    let _builder = Builder::default().a(12).b("some string".to_string());
}
