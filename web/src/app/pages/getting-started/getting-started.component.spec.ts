import "../../../test-setup";
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { GettingStartedComponent } from './getting-started.component';

describe('GettingStartedComponent', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [GettingStartedComponent],
      providers: [provideRouter([])],
    }).compileComponents();
  });

  it('should create', () => {
    const fixture = TestBed.createComponent(GettingStartedComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
  });

  it('should be a standalone component', () => {
    const fixture = TestBed.createComponent(GettingStartedComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
    expect(typeof component).toBe('object');
  });
});

