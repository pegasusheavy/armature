// React Server-Side Rendering integration for Armature

mod config;
mod renderer;
mod service;

pub use config::ReactConfig;
pub use renderer::ReactRenderer;
pub use service::ReactService;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
