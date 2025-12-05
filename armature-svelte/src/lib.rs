// Svelte Server-Side Rendering integration for Armature

mod config;
mod renderer;
mod service;

pub use config::SvelteConfig;
pub use renderer::SvelteRenderer;
pub use service::SvelteService;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
