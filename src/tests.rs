use super::*;

fn mock_getter(key: &str) -> Result<String, std::env::VarError> {
    match key {
        "STRING" => Ok("Hello".to_string()),
        "INT_OK" => Ok("42".to_string()),
        "INT_BAD" => Ok("not_an_int".to_string()),
        "MISSING" => Err(std::env::VarError::NotPresent),
        _ => panic!("Unexpected key"),
    }
}

#[test]
fn test_owned_var_try_ok() {
    let getter = EnvGetter::init_env(mock_getter);
    let result: Result<i32, _> = getter.owned_var_try("INT_OK");
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_owned_var_try_missing() {
    let getter = EnvGetter::init_env(mock_getter);
    let result: Result<i32, _> = getter.owned_var_try("MISSING");
    assert!(matches!(result, Err(EnvError::GetterError(_))));
}

#[test]
fn test_owned_var_try_parse_fail() {
    let getter = EnvGetter::init_env(mock_getter);
    let result: Result<i32, _> = getter.owned_var_try("INT_BAD");
    assert!(matches!(result, Err(EnvError::ParseError(_))));
}

#[test]
fn test_owned_var_or_default_used() {
    let getter = EnvGetter::init_env(mock_getter);
    let val: i32 = getter.owned_var_or("MISSING", 123);
    assert_eq!(val, 123);
}

#[test]
fn test_owned_var_or_default_skipped() {
    let getter = EnvGetter::init_env(mock_getter);
    let val: i32 = getter.owned_var_or("INT_OK", 123);
    assert_eq!(val, 42);
}

#[test]
fn test_owned_var_or_else_called() {
    let getter = EnvGetter::init_env(mock_getter);
    let val: i32 = getter.owned_var_or_else("MISSING", || 999);
    assert_eq!(val, 999);
}

#[test]
#[should_panic(expected = "Couldn't find or parse env variable INT_BAD")]
fn test_owned_var_panics_on_parse_error() {
    let getter = EnvGetter::init_env(mock_getter);
    let _: i32 = getter.owned_var("INT_BAD");
}

#[test]
fn test_var_try_leak() {
    let getter = EnvGetter::init_env(mock_getter);
    let val: &'static i32 = getter.var_try("INT_OK").unwrap();
    assert_eq!(*val, 42);
}

#[test]
fn test_var_or_else_leak() {
    let getter = EnvGetter::init_env(mock_getter);
    let val: &'static i32 = getter.var_or_else("MISSING", || 999);
    assert_eq!(*val, 999);
}

#[derive(Debug)]
struct DummyEnv {
    some_int: i32,
    some_other_int: i32,
    opt_int: Option<i32>,
    some_str: &'static str,
    owned_str: String
}

impl Env for DummyEnv {
    fn new() -> Self {
        let g = EnvGetter::init_env(mock_getter);

        Self {
            some_int: g.owned_var("INT_OK"),
            some_other_int: g.owned_var_or("INT_BAD", 100),
            opt_int: g.owned_var_try("MISSING").ok(),
            some_str: g.var::<String>("STRING").as_str(),
            owned_str: g.owned_var("STRING"),
        }
    }
}

#[test]
fn test_env_lazy_deref() {
    let env = EnvLazy::<DummyEnv>::new();
    let init: &DummyEnv = &*env;

    assert_eq!(init.some_int, 42);
    assert_eq!(init.some_other_int, 100);
    assert_eq!(init.opt_int, None);
    assert_eq!(init.some_str, "Hello");
    assert_eq!(init.owned_str, "Hello");
}

#[test]
fn test_env_once_deref_after_init() {
    let env = EnvOnce::<DummyEnv>::new();
    env.init();
    let init: &DummyEnv = &*env;

    assert_eq!(init.some_int, 42);
    assert_eq!(init.some_other_int, 100);
    assert_eq!(init.opt_int, None);
    assert_eq!(init.some_str, "Hello");
    assert_eq!(init.owned_str, "Hello");
}

#[test]
#[should_panic(expected = "Environment not initialized")]
fn test_env_once_deref_without_init_panics() {
    let env = EnvOnce::<DummyEnv>::new();
    let _ = &*env;
}

#[test]
#[should_panic(expected = "Failed to initialize environment")]
fn test_env_once_double_init_panics() {
    let env = EnvOnce::<DummyEnv>::new();
    env.init();
    env.init(); // Should panic on second init
}
