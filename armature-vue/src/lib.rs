// Vue.js Server-Side Rendering integration for Armature

mod config;
mod renderer;
mod service;

pub use config::VueConfig;
pub use renderer::VueRenderer;
pub use service::VueService;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
