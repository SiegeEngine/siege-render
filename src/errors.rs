error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
    }

    foreign_links {
    }

    errors {
        General(s: String) {
            description("General Error"),
            display("General Error: '{}'", s),
        }
    }
}
