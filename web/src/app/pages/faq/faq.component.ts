import { Component } from '@angular/core';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-faq',
  imports: [CommonModule, FontAwesomeModule],
  templateUrl: './faq.component.html',
  styleUrl: './faq.component.scss',
})
export class FaqComponent {}
