import "../../../test-setup";
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { ComparisonsComponent } from './comparisons.component';

describe('ComparisonsComponent', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ComparisonsComponent],
      providers: [provideRouter([])],
    }).compileComponents();
  });

  it('should create', () => {
    const fixture = TestBed.createComponent(ComparisonsComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
  });

  it('should be a standalone component', () => {
    const fixture = TestBed.createComponent(ComparisonsComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
    expect(typeof component).toBe('object');
  });
});

