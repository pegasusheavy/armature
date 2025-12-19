import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';

@Component({
  selector: 'app-grafana-dashboards',
  standalone: true,
  imports: [CommonModule, RouterModule],
  templateUrl: './grafana-dashboards.component.html',
  styleUrls: ['./grafana-dashboards.component.scss']
})
export class GrafanaDashboardsComponent {
  dashboards = [
    {
      name: 'Application Overview',
      file: 'armature-overview.json',
      description: 'High-level health and performance metrics',
      icon: '🌐'
    },
    {
      name: 'Auth & Security',
      file: 'armature-auth-security.json',
      description: 'Authentication flows and security monitoring',
      icon: '🔐'
    },
    {
      name: 'Cache & Redis',
      file: 'armature-cache-redis.json',
      description: 'Caching effectiveness and Redis performance',
      icon: '⚡'
    },
    {
      name: 'Queues & Jobs',
      file: 'armature-queues-jobs.json',
      description: 'Background job processing and cron tasks',
      icon: '📋'
    }
  ];

  alertThresholds = [
    { metric: 'P95 Latency', warning: '> 100ms', critical: '> 500ms' },
    { metric: 'Error Rate', warning: '> 1%', critical: '> 5%' },
    { metric: 'CPU Usage', warning: '> 70%', critical: '> 90%' },
    { metric: 'Memory Usage', warning: '> 70%', critical: '> 90%' },
    { metric: 'Cache Hit Rate', warning: '< 90%', critical: '< 70%' },
    { metric: 'Queue Depth', warning: '> 100', critical: '> 500' },
    { metric: 'Job Failure Rate', warning: '> 1%', critical: '> 5%' },
    { metric: 'Dead Letter Queue', warning: '> 10', critical: '> 50' }
  ];
}
