import "../../../test-setup";
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { NavComponent } from './nav.component';

describe('NavComponent', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [NavComponent],
      providers: [provideRouter([])],
    }).compileComponents();
  });

  it('should create', () => {
    const fixture = TestBed.createComponent(NavComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
  });

  it('should initialize with menu closed', () => {
    const fixture = TestBed.createComponent(NavComponent);
    const component = fixture.componentInstance;
    expect(component.isMenuOpen).toBe(false);
  });

  it('should toggle menu', () => {
    const fixture = TestBed.createComponent(NavComponent);
    const component = fixture.componentInstance;
    
    component.toggleMenu();
    expect(component.isMenuOpen).toBe(true);
    
    component.toggleMenu();
    expect(component.isMenuOpen).toBe(false);
  });

  it('should have menu state property', () => {
    const fixture = TestBed.createComponent(NavComponent);
    const component = fixture.componentInstance;
    expect(typeof component.isMenuOpen).toBe('boolean');
  });
});

