import '../test-setup';
import { routes } from './app.routes';
import { HomeComponent } from './pages/home/home.component';
import { DocsComponent } from './pages/docs/docs.component';
import { ComparisonsComponent } from './pages/comparisons/comparisons.component';
import { GettingStartedComponent } from './pages/getting-started/getting-started.component';
import { FaqComponent } from './pages/faq/faq.component';

describe('App Routes', () => {
  it('should have routes defined', () => {
    expect(routes).toBeDefined();
    expect(routes.length).toBeGreaterThan(0);
  });

  it('should have home route', () => {
    const homeRoute = routes.find((r) => r.path === '');
    expect(homeRoute).toBeDefined();
    expect(homeRoute?.component).toBe(HomeComponent);
  });

  it('should have comparisons route', () => {
    const comparisonsRoute = routes.find((r) => r.path === 'comparisons');
    expect(comparisonsRoute).toBeDefined();
    expect(comparisonsRoute?.component).toBe(ComparisonsComponent);
  });

  it('should have getting-started route', () => {
    const gettingStartedRoute = routes.find((r) => r.path === 'getting-started');
    expect(gettingStartedRoute).toBeDefined();
    expect(gettingStartedRoute?.component).toBe(GettingStartedComponent);
  });

  it('should have docs route', () => {
    const docsRoute = routes.find((r) => r.path === 'docs' && !r.path.includes(':'));
    expect(docsRoute).toBeDefined();
    expect(docsRoute?.component).toBe(DocsComponent);
  });

  it('should have dynamic docs route', () => {
    const docsIdRoute = routes.find((r) => r.path === 'docs/:id');
    expect(docsIdRoute).toBeDefined();
    expect(docsIdRoute?.component).toBe(DocsComponent);
  });

  it('should have faq route', () => {
    const faqRoute = routes.find((r) => r.path === 'faq');
    expect(faqRoute).toBeDefined();
    expect(faqRoute?.component).toBe(FaqComponent);
  });

  it('should have wildcard redirect', () => {
    const wildcardRoute = routes.find((r) => r.path === '**');
    expect(wildcardRoute).toBeDefined();
    expect(wildcardRoute?.redirectTo).toBe('');
  });

  it('should have 7 routes total', () => {
    expect(routes.length).toBe(7);
  });
});


