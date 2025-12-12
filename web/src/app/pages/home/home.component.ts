import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';
import { CommonModule } from '@angular/common';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';

@Component({
  selector: 'app-home',
  imports: [CommonModule, RouterLink, FontAwesomeModule],
  templateUrl: './home.component.html',
  styleUrl: './home.component.scss',
})
export class HomeComponent {
  features = [
    {
      icon: 'shield-alt',
      title: 'Type-Safe',
      description: 'Full Rust type safety with compile-time guarantees.',
      gradient: 'from-blue-500 to-cyan-500',
    },
    {
      icon: 'bolt',
      title: 'High Performance',
      description: 'Built on Tokio async runtime. Zero-cost abstractions.',
      gradient: 'from-amber-500 to-orange-500',
    },
    {
      icon: 'user-shield',
      title: 'Built-in Security',
      description: 'JWT, OAuth2, SAML auth and security middleware.',
      gradient: 'from-green-500 to-emerald-500',
    },
    {
      icon: 'server',
      title: 'Observability',
      description: 'OpenTelemetry integration for tracing and metrics.',
      gradient: 'from-indigo-500 to-violet-500',
    },
    {
      icon: 'check-circle',
      title: 'Validation',
      description: '18+ validators with custom rule builders.',
      gradient: 'from-teal-500 to-cyan-500',
    },
    {
      icon: 'rocket',
      title: 'Background Jobs',
      description: 'Redis-backed queues with retries and scheduling.',
      gradient: 'from-rose-500 to-pink-500',
    },
    {
      icon: 'terminal',
      title: 'Powerful CLI',
      description: 'Code generation and hot reloading tools.',
      gradient: 'from-slate-600 to-slate-800',
    },
    {
      icon: 'cubes',
      title: 'Project Templates',
      description: 'Start with minimal or full-featured templates.',
      gradient: 'from-purple-500 to-pink-500',
    },
  ];
}
