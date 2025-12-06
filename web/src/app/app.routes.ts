import { Routes } from '@angular/router';
import { HomeComponent } from './pages/home/home.component';
import { ComparisonsComponent } from './pages/comparisons/comparisons.component';
import { GettingStartedComponent } from './pages/getting-started/getting-started.component';
import { FaqComponent } from './pages/faq/faq.component';
import { DocsComponent } from './pages/docs/docs.component';

export const routes: Routes = [
  { path: '', component: HomeComponent, title: 'Armature - Modern Rust Web Framework' },
  {
    path: 'comparisons',
    component: ComparisonsComponent,
    title: 'Framework Comparisons - Armature',
  },
  {
    path: 'getting-started',
    component: GettingStartedComponent,
    title: 'Getting Started - Armature',
  },
  { path: 'docs', component: DocsComponent, title: 'Documentation - Armature' },
  { path: 'docs/:id', component: DocsComponent, title: 'Documentation - Armature' },
  { path: 'faq', component: FaqComponent, title: 'FAQ - Armature' },
  { path: '**', redirectTo: '', pathMatch: 'full' },
];
