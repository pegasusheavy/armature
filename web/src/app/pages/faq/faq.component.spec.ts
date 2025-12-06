import "../../../test-setup";
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { FaqComponent } from './faq.component';

describe('FaqComponent', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FaqComponent],
      providers: [provideRouter([])],
    }).compileComponents();
  });

  it('should create', () => {
    const fixture = TestBed.createComponent(FaqComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
  });

  it('should be a standalone component', () => {
    const fixture = TestBed.createComponent(FaqComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
    expect(typeof component).toBe('object');
  });
});

