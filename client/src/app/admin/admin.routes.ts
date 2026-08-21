import { Routes } from '@angular/router';
/* jhipster-needle-add-admin-module-import - JHipster will add admin modules imports here */

const routes: Routes = [
  {
    path: 'docs',
    loadComponent: () => import('./docs/docs'),
    title: 'global.menu.admin.apidocs',
  },
  /* jhipster-needle-add-admin-route - JHipster will add admin routes here */
];

export default routes;
