
# EnvInit

This is a small crate to help initializing environment variable structs,
accepting any environment provider and by default automatically parsing the
strings into their corresponding types in the struct.

## Example Usage with dotenvy

.env
```text
SOME_INT=42
SOME_STR=Hello
SOME_OTHER_STR=World
```

main.rs
```rust,ignore
use env_init::{Env, EnvGetter, EnvOnce};

#[derive(Debug)]
pub struct MyEnv {
    some_int: i32,
    some_other_int: i32,
    opt_int: Option<i32>,
    some_str: &'static str,
    owned_str: String
}

impl Env for MyEnv {
    fn new() -> Self {
        // Defining a getter that calls dotenvy::var
        let g = EnvGetter::new(|x| dotenvy::var(x));

        Self {
            some_int: g.owned_var("SOME_INT"),
            some_other_int: g.owned_var_or("SOME_OTHER_INT", 100),
            opt_int: g.owned_var_try("SOME_OPTIONAL_INT").ok(),
            some_str: g.var::<String>("SOME_STR").as_str(),
            owned_str: g.owned_var("SOME_OTHER_STR"),
        }
    }
}

// Our global environment object
pub static ENV: EnvOnce<MyEnv> = EnvOnce::new();

fn main() {
    dotenvy::dotenv().unwrap();
    ENV.init();

    assert_eq!(ENV.some_int, 42);
    assert_eq!(ENV.some_other_int, 100);
    assert_eq!(ENV.opt_int, None);
    assert_eq!(ENV.some_str, "Hello");
    assert_eq!(ENV.owned_str, "World");
}
```
