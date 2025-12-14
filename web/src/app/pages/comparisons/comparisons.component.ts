import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';

interface BenchmarkData {
  name: string;
  color: string;
  runtime: string;
  plaintextRps: number;
  jsonRps: number;
  pathParamRps: number;
  postRps: number;
  latencyP50: number;
  latencyP99: number;
  memoryIdle: number;
  memoryLoad: number;
}

@Component({
  selector: 'app-comparisons',
  imports: [CommonModule, FontAwesomeModule],
  templateUrl: './comparisons.component.html',
  styleUrl: './comparisons.component.scss',
})
export class ComparisonsComponent {
  activeTab: 'rust' | 'nodejs' | 'all' = 'all';

  rustFrameworks: BenchmarkData[] = [
    {
      name: 'Actix-web',
      color: '#f97316',
      runtime: 'Rust/Tokio',
      plaintextRps: 500000,
      jsonRps: 375000,
      pathParamRps: 320000,
      postRps: 280000,
      latencyP50: 0.08,
      latencyP99: 0.5,
      memoryIdle: 8,
      memoryLoad: 25,
    },
    {
      name: 'Axum',
      color: '#8b5cf6',
      runtime: 'Rust/Tokio',
      plaintextRps: 425000,
      jsonRps: 340000,
      pathParamRps: 290000,
      postRps: 250000,
      latencyP50: 0.1,
      latencyP99: 0.6,
      memoryIdle: 10,
      memoryLoad: 30,
    },
    {
      name: 'Warp',
      color: '#06b6d4',
      runtime: 'Rust/Tokio',
      plaintextRps: 375000,
      jsonRps: 300000,
      pathParamRps: 260000,
      postRps: 220000,
      latencyP50: 0.12,
      latencyP99: 0.8,
      memoryIdle: 9,
      memoryLoad: 28,
    },
    {
      name: 'Armature',
      color: '#b7410e',
      runtime: 'Rust/Tokio',
      plaintextRps: 325000,
      jsonRps: 225000,
      pathParamRps: 185000,
      postRps: 130000,
      latencyP50: 0.15,
      latencyP99: 1.2,
      memoryIdle: 12,
      memoryLoad: 35,
    },
    {
      name: 'Rocket',
      color: '#ef4444',
      runtime: 'Rust/Tokio',
      plaintextRps: 275000,
      jsonRps: 200000,
      pathParamRps: 170000,
      postRps: 120000,
      latencyP50: 0.18,
      latencyP99: 1.5,
      memoryIdle: 15,
      memoryLoad: 40,
    },
  ];

  nodejsFrameworks: BenchmarkData[] = [
    {
      name: 'Koa',
      color: '#22c55e',
      runtime: 'Node.js',
      plaintextRps: 42500,
      jsonRps: 37500,
      pathParamRps: 35000,
      postRps: 29000,
      latencyP50: 2.5,
      latencyP99: 8,
      memoryIdle: 32,
      memoryLoad: 90,
    },
    {
      name: 'Express',
      color: '#fbbf24',
      runtime: 'Node.js',
      plaintextRps: 37500,
      jsonRps: 32500,
      pathParamRps: 29000,
      postRps: 25000,
      latencyP50: 3,
      latencyP99: 10,
      memoryIdle: 40,
      memoryLoad: 115,
    },
    {
      name: 'NestJS',
      color: '#e11d48',
      runtime: 'Node.js',
      plaintextRps: 32500,
      jsonRps: 29000,
      pathParamRps: 25000,
      postRps: 21000,
      latencyP50: 3.5,
      latencyP99: 12,
      memoryIdle: 65,
      memoryLoad: 150,
    },
    {
      name: 'Next.js',
      color: '#000000',
      runtime: 'Node.js',
      plaintextRps: 27500,
      jsonRps: 23500,
      pathParamRps: 20000,
      postRps: 16500,
      latencyP50: 4,
      latencyP99: 15,
      memoryIdle: 85,
      memoryLoad: 180,
    },
  ];

  get displayedFrameworks(): BenchmarkData[] {
    if (this.activeTab === 'rust') return this.rustFrameworks;
    if (this.activeTab === 'nodejs') return this.nodejsFrameworks;
    return [...this.rustFrameworks, ...this.nodejsFrameworks];
  }

  get maxRps(): number {
    return Math.max(...this.displayedFrameworks.map((f) => f.plaintextRps));
  }

  getBarWidth(value: number, max: number): number {
    return (value / max) * 100;
  }

  formatNumber(num: number): string {
    if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
    if (num >= 1000) return (num / 1000).toFixed(0) + 'K';
    return num.toString();
  }

  getRatioVsArmature(framework: BenchmarkData): string {
    const armature = this.rustFrameworks.find((f) => f.name === 'Armature');
    if (!armature || framework.name === 'Armature') return '-';

    const ratio = framework.plaintextRps / armature.plaintextRps;
    if (ratio > 1) {
      return `${ratio.toFixed(1)}x faster`;
    } else {
      return `${(1 / ratio).toFixed(1)}x slower`;
    }
  }
}
