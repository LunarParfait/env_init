#![doc = include_str!("../README.md")]

use std::ops::Deref;
use std::str::FromStr;
use std::sync::{LazyLock, OnceLock};

#[cfg(test)]
mod tests;

/// Struct that holds a closure to get environment variables.
///
/// # Examples
/// ```
/// use env_init::{EnvGetter, EnvError};
///
/// let closure = |x: &str| {
///     if x == "KEY" {
///         Ok("VALUE".to_string())
///     } else {
///         Err(())
///     }
/// };
///
/// let g = EnvGetter::new(closure);
/// let key = g.owned_var_try::<String>("KEY");
/// let not_found = g.owned_var_try::<String>("NOT_FOUND");
///
/// assert_eq!(key, Ok("VALUE".to_string()));
/// assert_eq!(not_found, Err(EnvError::GetterError(())));
/// ```
pub struct EnvGetter<G, I: Fn(&str) -> Result<String, G>> {
    getter: I,
}

// TODO: remove thiserror dependency

/// Custom error type, which can be a GetterError (error returned by
/// closure) or ParseError (error returned by [`FromStr`]).
#[derive(thiserror::Error, Debug, Clone, PartialEq, PartialOrd)]
pub enum EnvError<G, P> {
    GetterError(G),
    ParseError(P),
}

impl<G, I: Fn(&str) -> Result<String, G>> EnvGetter<G, I> {
    /// Initializes a new [`EnvGetter`] from a closure
    pub fn new(getter: I) -> Self {
        Self { getter }
    }

    /// Useful when you want to handle the Result yourself, and do not want the
    /// result to be leaked.
    ///
    /// The leaking version of this is [`Self::var_try`].
    ///
    /// # Errors
    /// When the environment variable is not found or when the parsing fails for R.
    pub fn owned_var_try<T: FromStr>(
        &self,
        name: &str,
    ) -> Result<T, EnvError<G, T::Err>> {
        let var = (self.getter)(name).map_err(EnvError::GetterError)?;
        var.parse::<T>().map_err(EnvError::ParseError)
    }

    /// Useful when your program requires a variable to be defined and cannot provide a
    /// default alternative, but you do not want the parsed result to be leaked/static ref.
    /// E.g.: Any Copy type. Not worth leaking.
    ///
    /// The leaking version of this is [`Self::var`].
    ///
    /// # Panics
    /// When the environment variable is not found or when the parsing fails for T.
    pub fn owned_var<T: FromStr>(&self, name: &str) -> T {
        self.owned_var_try(name).unwrap_or_else(|_| {
            panic!("Couldn't find or parse env variable {name} for given type")
        })
    }

    /// Useful when you want to provide a default value for the environment variable,
    /// but you do not want the parsed result to be leaked or static.
    /// E.g.: Any Copy type. Not worth leaking.
    ///
    /// The leaking version of this function is [`Self::var_or`].
    pub fn owned_var_or<T: FromStr>(&self, name: &str, default: T) -> T {
        self.owned_var_try(name).unwrap_or(default)
    }

    /// Useful when you want to provide a default value for the environment variable,
    /// but you do not want the parsed result to be leaked or static. Use this over
    /// [`Self::owned_var_or`] when you need to provide a closure for the default value.
    ///
    /// The leaking version of this function is [`Self::var_or_else`].
    pub fn owned_var_or_else<T: FromStr, V: FnOnce() -> T>(
        &self,
        name: &str,
        default: V,
    ) -> T {
        self.owned_var_try(name).unwrap_or_else(|_| default())
    }

    /// Utility to attempt leaking a Box to your desired static reference type.
    fn leak<T>(to_leak: T) -> &'static T {
        Box::leak(Box::new(to_leak))
    }

    /// Useful when you want to handle the Result yourself.
    ///
    /// # Leaks
    /// This function will leak the parsed value, if any.
    ///
    /// # Errors
    /// This function will error if it fails to parse the value, or the environment variable
    /// is not found
    pub fn var_try<T: FromStr>(
        &self,
        name: &str,
    ) -> Result<&'static T, EnvError<G, T::Err>> {
        self.owned_var_try::<T>(name).map(Self::leak)
    }

    /// Useful when your program requires a variable to be defined and cannot
    /// provide a default alternative.
    ///
    /// # Leaks
    /// This function will leak the parsed value.
    ///
    /// # Panics
    /// When the environment variable is not found or when the parsing fails for R.
    pub fn var<T: FromStr>(&self, name: &str) -> &'static T {
        self.var_try(name).unwrap_or_else(|_| {
            panic!("Couldn't find or parse env variable {name} for given type")
        })
    }

    /// Useful when you want to provide a default value for the environment variable,
    /// and you have a static reference to your default value.
    /// E.g.: A string literal that is stored in the binary.
    ///
    /// # Leaks
    /// This function will leak the parsed value.
    pub fn var_or<T: FromStr>(
        &self,
        name: &str,
        default: &'static T,
    ) -> &'static T {
        self.var_try(name).unwrap_or(default)
    }

    /// Useful when you want to provide a default value for the environment variable,
    /// but you don't have a static reference to the value.
    /// E.g.: An owned [`PathBuf`](std::path::PathBuf) -> A `&'static Path`.
    ///
    /// # Leaks
    /// This function will leak the parsed or the default value.
    pub fn var_or_else<T: FromStr, V: FnOnce() -> T>(
        &self,
        name: &str,
        default: V,
    ) -> &'static T {
        self.var_or(name, Box::leak(default().into()))
    }
}

/// This trait is used to create a new environment struct.
pub trait Env {
    fn new() -> Self;
}

/// Wrapper over a [`LazyLock<T>`] where T implements [`Env`]
pub struct EnvLazy<T: Env>(LazyLock<T>);

impl<T: Env> EnvLazy<T> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(LazyLock::new(T::new))
    }
}

impl<T: Env> Deref for EnvLazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: Env> AsRef<T> for EnvLazy<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

/// Wrapper over a [`OnceLock<T>`] where T implements [`Env`]
pub struct EnvOnce<T: Env>(OnceLock<T>);

impl<T: Env> EnvOnce<T> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn init(&self) {
        self.0
            .set(T::new())
            .unwrap_or_else(|_| panic!("Failed to initialize environment"));
    }
}

impl<T: Env> Deref for EnvOnce<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
            .get()
            .unwrap_or_else(|| panic!("Environment not initialized"))
    }
}

impl<T: Env> AsRef<T> for EnvOnce<T> {
    fn as_ref(&self) -> &T {
        self
    }
}
