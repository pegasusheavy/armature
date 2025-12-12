# Armature Framework - Web Documentation

This is the official documentation website for the Armature Framework, built with Angular 21+ and Tailwind CSS 4+.

## About Armature

**Armature** is a modern, type-safe HTTP framework for Rust that brings the best architectural patterns from **NestJS**, **Express**, and **Koa** to the Rust ecosystem. Perfect for:

- 🔄 **Node.js developers** migrating to Rust (NestJS, Express, Koa, Fastify users)
- 🦀 **Rust developers** seeking higher-level abstractions (Actix-web, Rocket, Axum alternatives)
- 🏢 **Enterprise teams** needing built-in auth, validation, caching, and observability

### Framework Comparisons

| Feature | Armature | NestJS | Express | Actix-web | Rocket |
|---------|----------|--------|---------|-----------|--------|
| Language | Rust | TypeScript | JavaScript | Rust | Rust |
| DI Container | ✅ | ✅ | ❌ | ❌ | ❌ |
| Decorators | ✅ | ✅ | ❌ | ❌ | ✅ |
| Type Safety | Compile-time | Runtime | None | Compile-time | Compile-time |
| Built-in Auth | ✅ | ✅ | ❌ | ❌ | ❌ |
| OpenAPI | ✅ | ✅ | ❌ | ❌ (utoipa) | ❌ (okapi) |
| Rate Limiting | ✅ | ✅ | ❌ | ❌ | ❌ |

### Key Features

- 🎯 **Familiar Patterns**: Decorators, dependency injection, modules (like NestJS)
- 🚀 **High Performance**: Native Rust performance and memory safety
- 🛡️ **Type Safety**: Compile-time guarantees, not runtime checks
- 🔐 **Built-in Auth**: JWT, OAuth2, SAML support out of the box
- 📚 **OpenAPI/Swagger**: Automatic API documentation generation
- ⚡ **Rate Limiting**: Multiple algorithms with Redis support
- 💼 **Enterprise Ready**: Caching, queues, validation, observability

## Technology Stack

- **Angular**: 21.0.1 (latest)
- **Tailwind CSS**: 4.1.17 (CSS-first configuration)
- **SCSS**: For enhanced styling capabilities
- **TypeScript**: 5.7+
- **Package Manager**: pnpm

## Tailwind CSS 4+ Configuration

This project uses Tailwind CSS 4's new CSS-first configuration approach. No more `tailwind.config.js`!

### How it works:

1. **Direct CSS Import**: Tailwind is imported directly in `src/styles.scss`:
   ```scss
   @import "tailwindcss";
   ```

2. **Theme Configuration**: Custom themes are defined using `@theme` directive with CSS variables:
   ```scss
   @theme {
     --color-primary-500: #0ea5e9;
     --font-family-sans: Inter, system-ui, sans-serif;
   }
   ```

3. **Utility Classes**: Use Tailwind utilities directly in templates with `@apply` in SCSS.

## Development

```bash
# Install dependencies
pnpm install

# Start development server
pnpm start

# Build for production
pnpm run build

# Run tests
pnpm test

# Run linting
pnpm lint
```

## Project Structure

```
web/
├── src/
│   ├── app/
│   │   ├── app.ts          # Main component
│   │   ├── app.html        # Component template
│   │   ├── app.scss        # Component styles
│   │   └── app.config.ts   # App configuration
│   ├── styles.scss         # Global styles + Tailwind
│   ├── index.html          # HTML entry point
│   └── main.ts             # TypeScript entry point
├── public/
│   └── favicon.ico
└── angular.json            # Angular configuration
```

## Features

- ✅ Standalone Components (Angular 21+)
- ✅ Tailwind CSS 4+ with CSS-first configuration
- ✅ SCSS for enhanced styling
- ✅ Responsive design with mobile-first approach
- ✅ TypeScript with strict mode
- ✅ Production-ready build optimization
- ✅ Modern CSS features (CSS Grid, Flexbox)
- ✅ Google Fonts integration (Inter)

## Customizing Styles

### Adding Custom Colors

Edit `src/styles.scss`:

```scss
@theme {
  --color-accent-500: #f59e0b;
  --color-accent-600: #d97706;
}
```

Then use in templates:

```html
<div class="bg-accent-500 text-white">Custom color</div>
```

### Component-Specific Styles

Add styles to `src/app/app.scss` using Tailwind's `@apply`:

```scss
.my-custom-class {
  @apply px-4 py-2 bg-primary-500 text-white rounded-lg;
}
```

## Deployment

### Build for Production

```bash
pnpm run build
```

Output will be in `dist/web/`.

### Deploy to GitHub Pages

The site is configured for automatic deployment via GitHub Actions:

```bash
# Commit changes to main branch
git add .
git commit -m "Update web documentation"

# Push to main
git push origin main
```

GitHub Pages will automatically build and deploy the site when changes are pushed to the `main` branch.

Live site: https://pegasusheavy.github.io/armature/

## Browser Support

- Chrome/Edge (last 2 versions)
- Firefox (last 2 versions)
- Safari (last 2 versions)
- Mobile browsers (iOS Safari, Chrome Mobile)

## License

MIT License - See root LICENSE file for details.

## Contributing

See root CONTRIBUTING.md for contribution guidelines.
