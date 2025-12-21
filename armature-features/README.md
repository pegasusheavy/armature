# armature-features

Feature flags and A/B testing for the Armature framework.

## Features

- **Feature Flags** - Toggle features on/off
- **A/B Testing** - Experiment variants
- **Gradual Rollout** - Percentage-based rollouts
- **User Targeting** - Target specific users

## Installation

```toml
[dependencies]
armature-features = "0.1"
```

## Quick Start

```rust
use armature_features::FeatureFlags;

let flags = FeatureFlags::new()
    .launchdarkly("sdk-key")  // or custom backend
    .build();

// Check feature
if flags.is_enabled("new-checkout", &user_context) {
    // New checkout flow
}

// Get variant
let variant = flags.get_variant("button-color", &user_context);
```

## License

MIT OR Apache-2.0

