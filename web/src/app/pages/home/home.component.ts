import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';
import { CommonModule } from '@angular/common';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';

interface Feature {
  icon: string;
  title: string;
  description: string;
  gradient: string;
}

@Component({
  selector: 'app-home',
  imports: [CommonModule, RouterLink, FontAwesomeModule],
  templateUrl: './home.component.html',
  styleUrl: './home.component.scss',
})
export class HomeComponent {
  currentSlide = 0;
  slidesPerView = 3;

  features: Feature[] = [
    {
      icon: 'shield-alt',
      title: 'Type-Safe',
      description: 'Full Rust type safety with compile-time guarantees. Catch errors before production.',
      gradient: 'from-blue-500 to-cyan-500',
    },
    {
      icon: 'bolt',
      title: 'High Performance',
      description: 'Built on Tokio async runtime with zero-cost abstractions. Blazingly fast.',
      gradient: 'from-amber-500 to-orange-500',
    },
    {
      icon: 'project-diagram',
      title: 'Modular Architecture',
      description: 'Organize code into modules with dependency injection, like Angular and NestJS.',
      gradient: 'from-purple-500 to-pink-500',
    },
    {
      icon: 'user-shield',
      title: 'Built-in Security',
      description: 'JWT, OAuth2, SAML authentication and Helmet-like security middleware.',
      gradient: 'from-green-500 to-emerald-500',
    },
    {
      icon: 'server',
      title: 'Observability',
      description: 'OpenTelemetry integration with tracing, metrics, and structured logging.',
      gradient: 'from-indigo-500 to-violet-500',
    },
    {
      icon: 'check-circle',
      title: 'Validation',
      description: '18+ built-in validators with custom rule builders and async validation.',
      gradient: 'from-teal-500 to-cyan-500',
    },
    {
      icon: 'rocket',
      title: 'Background Jobs',
      description: 'Redis-backed queue system with retry logic, priorities, and scheduling.',
      gradient: 'from-rose-500 to-pink-500',
    },
    {
      icon: 'terminal',
      title: 'Powerful CLI',
      description: 'Code generation, hot reloading, and project scaffolding tools.',
      gradient: 'from-slate-600 to-slate-800',
    },
  ];

  get totalSlides(): number {
    return Math.ceil(this.features.length / this.slidesPerView);
  }

  get visibleFeatures(): Feature[] {
    const start = this.currentSlide * this.slidesPerView;
    return this.features.slice(start, start + this.slidesPerView);
  }

  nextSlide(): void {
    if (this.currentSlide < this.totalSlides - 1) {
      this.currentSlide++;
    }
  }

  prevSlide(): void {
    if (this.currentSlide > 0) {
      this.currentSlide--;
    }
  }

  goToSlide(index: number): void {
    this.currentSlide = index;
  }
}
