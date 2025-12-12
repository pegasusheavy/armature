import { Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { FaIconLibrary, FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import {
  faBolt,
  faShieldAlt,
  faProjectDiagram,
  faUserShield,
  faPalette,
  faServer,
  faCheckCircle,
  faRocket,
  faCode,
  faChevronDown,
  faCube,
  faBuilding,
  faDownload,
  faBook,
  faRecycle,
  faFileAlt,
  faTachometerAlt,
  faDatabase,
  faTerminal,
  faCubes,
  faMagic,
  faSyringe,
  faCheckDouble,
  faLayerGroup,
} from '@fortawesome/free-solid-svg-icons';
import { faGithub } from '@fortawesome/free-brands-svg-icons';

import { NavComponent } from './components/nav/nav.component';
import { FooterComponent } from './components/footer/footer.component';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, NavComponent, FooterComponent, FontAwesomeModule],
  templateUrl: './app.html',
  styleUrls: ['./app.scss'],
})
export class App {
  constructor(library: FaIconLibrary) {
    // Add solid icons
    library.addIcons(
      faBolt,
      faShieldAlt,
      faProjectDiagram,
      faUserShield,
      faPalette,
      faServer,
      faCheckCircle,
      faRocket,
      faCode,
      faChevronDown,
      faCube,
      faBuilding,
      faDownload,
      faBook,
      faRecycle,
      faFileAlt,
      faTachometerAlt,
      faDatabase,
      faTerminal,
      faCubes,
      faMagic,
      faSyringe,
      faCheckDouble,
      faLayerGroup
    );

    // Add brand icons
    library.addIcons(faGithub);
  }
}
