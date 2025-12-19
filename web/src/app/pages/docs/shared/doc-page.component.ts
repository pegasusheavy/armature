import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';

export interface CodeBlock {
  language: string;
  code: string;
  filename?: string;
}

export interface DocSection {
  id: string;
  title: string;
  content?: string;
  codeBlocks?: CodeBlock[];
  subsections?: DocSection[];
  type?: 'text' | 'code' | 'feature-grid' | 'table' | 'tip' | 'warning' | 'info';
}

export interface DocFeature {
  icon: string;
  title: string;
  description: string;
  link?: string;
}

export interface DocTableRow {
  [key: string]: string;
}

export interface DocPage {
  title: string;
  subtitle: string;
  icon?: string;
  badge?: string;
  features?: DocFeature[];
  sections: DocSection[];
  relatedDocs?: { title: string; id: string; description: string }[];
  seeAlso?: { title: string; id: string }[];
}

@Component({
  selector: 'app-doc-page',
  standalone: true,
  imports: [CommonModule, RouterModule],
  template: `
    <article class="doc-page">
      <!-- Hero Section -->
      <header class="doc-hero">
        <div class="hero-content">
          @if (page.icon) {
            <span class="hero-icon">{{ page.icon }}</span>
          }
          <div class="hero-text">
            <div class="title-row">
              <h1>{{ page.title }}</h1>
              @if (page.badge) {
                <span class="hero-badge">{{ page.badge }}</span>
              }
            </div>
            <p class="hero-subtitle">{{ page.subtitle }}</p>
          </div>
        </div>
      </header>

      <!-- Quick Feature Cards -->
      @if (page.features && page.features.length) {
        <section class="feature-cards">
          @for (feature of page.features; track feature.title) {
            <div class="feature-card">
              <span class="feature-icon">{{ feature.icon }}</span>
              <div class="feature-content">
                <h3>{{ feature.title }}</h3>
                <p>{{ feature.description }}</p>
                @if (feature.link) {
                  <a [routerLink]="['/docs', feature.link]" class="feature-link">Learn more →</a>
                }
              </div>
            </div>
          }
        </section>
      }

      <!-- Main content area with optional TOC sidebar -->
      <div class="doc-layout" [class.has-toc]="page.sections.length > 3">
        <!-- Content Sections -->
        <div class="doc-sections">
        @for (section of page.sections; track section.id) {
          <section [id]="section.id" class="doc-section" [class]="'section-' + (section.type || 'text')">
            <h2>{{ section.title }}</h2>

            @if (section.content) {
              <div class="section-content" [innerHTML]="section.content"></div>
            }

            @if (section.codeBlocks) {
              @for (code of section.codeBlocks; track code.code) {
                <div class="code-block-wrapper">
                  @if (code.filename) {
                    <div class="code-header">
                      <span class="code-filename">{{ code.filename }}</span>
                      <span class="code-lang">{{ code.language }}</span>
                    </div>
                  }
                  <pre class="code-block" [class]="'language-' + code.language"><code [innerHTML]="highlightCode(code.code, code.language)"></code></pre>
                </div>
              }
            }

            @if (section.subsections) {
              @for (sub of section.subsections; track sub.id) {
                <div [id]="sub.id" class="subsection">
                  <h3>{{ sub.title }}</h3>
                  @if (sub.content) {
                    <div class="subsection-content" [innerHTML]="sub.content"></div>
                  }
                  @if (sub.codeBlocks) {
                    @for (code of sub.codeBlocks; track code.code) {
                      <div class="code-block-wrapper">
                        @if (code.filename) {
                          <div class="code-header">
                            <span class="code-filename">{{ code.filename }}</span>
                            <span class="code-lang">{{ code.language }}</span>
                          </div>
                        }
                        <pre class="code-block" [class]="'language-' + code.language"><code [innerHTML]="highlightCode(code.code, code.language)"></code></pre>
                      </div>
                    }
                  }
                </div>
              }
            }
          </section>
        }
        </div>

        <!-- Table of Contents - Sticky Sidebar -->
        @if (page.sections.length > 3) {
          <aside class="table-of-contents">
            <h2>On This Page</h2>
            <ul>
              @for (section of page.sections; track section.id) {
                <li>
                  <a [href]="'#' + section.id">{{ section.title }}</a>
                  @if (section.subsections) {
                    <ul>
                      @for (sub of section.subsections; track sub.id) {
                        <li><a [href]="'#' + sub.id">{{ sub.title }}</a></li>
                      }
                    </ul>
                  }
                </li>
              }
            </ul>
          </aside>
        }
      </div>

      <!-- Related Documentation -->
      @if (page.relatedDocs && page.relatedDocs.length) {
        <section class="related-docs">
          <h2>Related Documentation</h2>
          <div class="related-grid">
            @for (doc of page.relatedDocs; track doc.id) {
              <a [routerLink]="['/docs', doc.id]" class="related-card">
                <h3>{{ doc.title }}</h3>
                <p>{{ doc.description }}</p>
                <span class="related-link">Read more →</span>
              </a>
            }
          </div>
        </section>
      }

      <!-- See Also -->
      @if (page.seeAlso && page.seeAlso.length) {
        <footer class="see-also">
          <h3>See Also</h3>
          <ul>
            @for (link of page.seeAlso; track link.id) {
              <li><a [routerLink]="['/docs', link.id]">{{ link.title }}</a></li>
            }
          </ul>
        </footer>
      }
    </article>
  `,
  styleUrls: ['./doc-page.component.scss']
})
export class DocPageComponent {
  @Input() page!: DocPage;

  constructor(private sanitizer: DomSanitizer) {}

  highlightCode(code: string, language: string): SafeHtml {
    // Basic syntax highlighting - can be enhanced with a library like highlight.js
    let highlighted = this.escapeHtml(code);

    if (language === 'rust') {
      // Keywords
      highlighted = highlighted.replace(
        /\b(use|mod|pub|fn|async|await|let|mut|const|struct|impl|trait|enum|match|if|else|for|while|loop|return|break|continue|where|type|self|Self|dyn|move|ref|as|in|extern|crate|super|unsafe)\b/g,
        '<span class="keyword">$1</span>'
      );
      // Types
      highlighted = highlighted.replace(
        /\b(String|Vec|Option|Result|Ok|Err|Some|None|Box|Arc|Rc|HashMap|HashSet|bool|i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64|usize|isize|str)\b/g,
        '<span class="type">$1</span>'
      );
      // Strings
      highlighted = highlighted.replace(
        /"([^"\\]|\\.)*"/g,
        '<span class="string">$&</span>'
      );
      // Numbers
      highlighted = highlighted.replace(
        /\b(\d+\.?\d*)\b/g,
        '<span class="number">$1</span>'
      );
      // Comments
      highlighted = highlighted.replace(
        /(\/\/.*$)/gm,
        '<span class="comment">$1</span>'
      );
      // Attributes
      highlighted = highlighted.replace(
        /(#\[[^\]]+\])/g,
        '<span class="attribute">$1</span>'
      );
      // Macros
      highlighted = highlighted.replace(
        /\b([a-z_]+!)\(/g,
        '<span class="macro">$1</span>('
      );
    } else if (language === 'toml') {
      // Section headers
      highlighted = highlighted.replace(
        /^\[([^\]]+)\]/gm,
        '<span class="section">[$1]</span>'
      );
      // Keys
      highlighted = highlighted.replace(
        /^(\w+)\s*=/gm,
        '<span class="key">$1</span> ='
      );
      // Strings
      highlighted = highlighted.replace(
        /"([^"\\]|\\.)*"/g,
        '<span class="string">$&</span>'
      );
    } else if (language === 'bash' || language === 'sh' || language === 'shell') {
      // Commands
      highlighted = highlighted.replace(
        /^(\$\s*)(.+)$/gm,
        '<span class="prompt">$1</span><span class="command">$2</span>'
      );
      // Comments
      highlighted = highlighted.replace(
        /(#.*$)/gm,
        '<span class="comment">$1</span>'
      );
    }

    return this.sanitizer.bypassSecurityTrustHtml(highlighted);
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }
}

