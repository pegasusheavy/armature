import "../../../test-setup";
import { TestBed } from '@angular/core/testing';
import { provideRouter, Router } from '@angular/router';
import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { DocsComponent } from './docs.component';
import { ActivatedRoute } from '@angular/router';
import { BehaviorSubject } from 'rxjs';

describe('DocsComponent', () => {
  let httpTesting: HttpTestingController;
  let paramMapSubject: BehaviorSubject<any>;

  beforeEach(async () => {
    paramMapSubject = new BehaviorSubject({
      get: (key: string) => null
    });

    await TestBed.configureTestingModule({
      imports: [DocsComponent],
      providers: [
        provideRouter([]), 
        provideHttpClient(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            paramMap: paramMapSubject.asObservable(),
            snapshot: {
              paramMap: {
                get: (key: string) => null
              }
            }
          }
        }
      ],
    }).compileComponents();

    httpTesting = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpTesting.verify();
  });

  it('should create', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
  });

  it('should initialize with loading state', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    expect(component.loading()).toBe(true);
  });

  it('should have documentation metadata', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    expect(component.docs.length).toBeGreaterThan(0);
  });

  it('should have Getting Started category docs', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const gettingStartedDocs = component.docs.filter(
      (doc) => doc.category === 'Getting Started'
    );
    expect(gettingStartedDocs.length).toBeGreaterThan(0);
  });

  it('should have Core Features category docs', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const coreFeaturesDocs = component.docs.filter(
      (doc) => doc.category === 'Core Features'
    );
    expect(coreFeaturesDocs.length).toBeGreaterThan(0);
  });

  it('should have SSR Frameworks category docs', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const ssrDocs = component.docs.filter((doc) => doc.category === 'SSR Frameworks');
    expect(ssrDocs.length).toBeGreaterThan(0);
  });

  it('should group docs by category', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const grouped = component.docsByCategory();
    expect(Object.keys(grouped).length).toBeGreaterThan(0);
  });

  it('should have readme doc', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const readmeDoc = component.docs.find((doc) => doc.id === 'readme');
    expect(readmeDoc).toBeTruthy();
    expect(readmeDoc?.title).toBe('Getting Started');
  });

  it('should have authentication guide', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const authDoc = component.docs.find((doc) => doc.id === 'auth-guide');
    expect(authDoc).toBeTruthy();
    expect(authDoc?.filename).toBe('AUTH_GUIDE.md');
  });

  it('should have lifecycle hooks guide', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const lifecycleDoc = component.docs.find((doc) => doc.id === 'lifecycle-hooks');
    expect(lifecycleDoc).toBeTruthy();
  });

  it('should have HMR guides', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const hmrGuide = component.docs.find((doc) => doc.id === 'hmr-guide');
    const hmrQuickStart = component.docs.find((doc) => doc.id === 'hmr-quick-start');
    expect(hmrGuide).toBeTruthy();
    expect(hmrQuickStart).toBeTruthy();
  });

  it('should have logging guide', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const loggingDoc = component.docs.find((doc) => doc.id === 'logging-guide');
    expect(loggingDoc).toBeTruthy();
  });

  it('should have parallel processing guide', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const parallelDoc = component.docs.find((doc) => doc.id === 'parallel-processing');
    expect(parallelDoc).toBeTruthy();
  });

  it('should return category list', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const categories = component.getCategories();
    expect(categories).toContain('Getting Started');
    expect(categories).toContain('Core Features');
    expect(categories).toContain('SSR Frameworks');
    expect(categories).toContain('Architecture');
  });

  it('should set currentDoc to null initially', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    expect(component.currentDoc()).toBeNull();
  });

  it('should set error to null initially', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    expect(component.error()).toBeNull();
  });

  it('should set content to empty string initially', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    expect(component.content()).toBe('');
  });

  it('should have 34 documentation entries', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    expect(component.docs.length).toBe(34);
  });

  it('should have all documentation categories', () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const grouped = component.docsByCategory();
    const expectedCategories = [
      'Getting Started',
      'Core Features',
      'HTTP & Networking',
      'SSR Frameworks',
      'GraphQL',
      'OpenAPI',
      'Background Processing',
      'Observability',
      'Architecture',
      'Testing & Quality',
      'Security',
    ];
    expectedCategories.forEach((category) => {
      expect(grouped[category]).toBeDefined();
      expect(grouped[category].length).toBeGreaterThan(0);
    });
  });

  it('should handle doc not found in loadDoc', async () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    
    await component.loadDoc('non-existent-doc');
    
    expect(component.error()).toBe('Documentation not found');
    expect(component.loading()).toBe(false);
  });

  it('should successfully load a document', async () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const mockMarkdown = '# Test Documentation\n\nThis is a test.';
    
    const loadPromise = component.loadDoc('readme');
    
    const req = httpTesting.expectOne('/docs/README.md');
    expect(req.request.method).toBe('GET');
    req.flush(mockMarkdown);
    
    await loadPromise;
    
    expect(component.currentDoc()?.id).toBe('readme');
    expect(component.error()).toBeNull();
    expect(component.loading()).toBe(false);
    expect(component.content()).toBeTruthy();
  });

  it('should handle HTTP error when loading document', async () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    
    const loadPromise = component.loadDoc('readme');
    
    const req = httpTesting.expectOne('/docs/README.md');
    req.error(new ProgressEvent('Network error'));
    
    await loadPromise;
    
    expect(component.error()).toBe('Failed to load documentation');
    expect(component.loading()).toBe(false);
    expect(consoleErrorSpy).toHaveBeenCalled();
    
    consoleErrorSpy.mockRestore();
  });

  it('should navigate to readme by default when no docId', async () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const router = TestBed.inject(Router);
    const navigateSpy = vi.spyOn(router, 'navigate');
    
    // Trigger ngOnInit by emitting paramMap without docId
    paramMapSubject.next({
      get: (key: string) => null
    });
    
    await fixture.whenStable();
    
    expect(navigateSpy).toHaveBeenCalledWith(['/docs/readme']);
  });

  it('should load document when docId is provided in route', async () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    const loadDocSpy = vi.spyOn(component, 'loadDoc').mockResolvedValue();
    
    // Trigger ngOnInit by emitting paramMap with docId
    paramMapSubject.next({
      get: (key: string) => key === 'id' ? 'auth-guide' : null
    });
    
    await fixture.whenStable();
    
    expect(loadDocSpy).toHaveBeenCalledWith('auth-guide');
  });

  it('should set currentDoc when loading', async () => {
    const fixture = TestBed.createComponent(DocsComponent);
    const component = fixture.componentInstance;
    
    const loadPromise = component.loadDoc('auth-guide');
    
    const req = httpTesting.expectOne('/docs/AUTH_GUIDE.md');
    req.flush('# Auth Guide');
    
    await loadPromise;
    
    const currentDoc = component.currentDoc();
    expect(currentDoc).toBeTruthy();
    expect(currentDoc?.id).toBe('auth-guide');
    expect(currentDoc?.filename).toBe('AUTH_GUIDE.md');
  });
});

