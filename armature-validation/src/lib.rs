// Validation framework for Armature

mod errors;
mod pipe;
mod rules;
mod traits;
mod validators;

pub use errors::*;
pub use pipe::*;
pub use rules::*;
pub use traits::*;
pub use validators::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
