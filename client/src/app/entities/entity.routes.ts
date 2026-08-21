import { Routes } from '@angular/router';

const routes: Routes = [
  {
    path: 'authority',
    data: { pageTitle: 'jobSearchRustApp.adminAuthority.home.title' },
    loadChildren: () => import('./admin/authority/authority.routes'),
  },
  {
    path: 'job-offer',
    data: { pageTitle: 'jobSearchRustApp.jobOffer.home.title' },
    loadChildren: () => import('./job-offer/job-offer.routes'),
  },
  {
    path: 'candidate-profile',
    data: { pageTitle: 'jobSearchRustApp.candidateProfile.home.title' },
    loadChildren: () => import('./candidate-profile/candidate-profile.routes'),
  },
  {
    path: 'job-application',
    data: { pageTitle: 'jobSearchRustApp.jobApplication.home.title' },
    loadChildren: () => import('./job-application/job-application.routes'),
  },
  {
    path: 'user-preference',
    data: { pageTitle: 'jobSearchRustApp.userPreference.home.title' },
    loadChildren: () => import('./user-preference/user-preference.routes'),
  },
  {
    path: 'auto-apply-config',
    data: { pageTitle: 'jobSearchRustApp.autoApplyConfig.home.title' },
    loadChildren: () => import('./auto-apply-config/auto-apply-config.routes'),
  },
  {
    path: 'radar-hit',
    data: { pageTitle: 'jobSearchRustApp.radarHit.home.title' },
    loadChildren: () => import('./radar-hit/radar-hit.routes'),
  },
  {
    path: 'radar-state',
    data: { pageTitle: 'jobSearchRustApp.radarState.home.title' },
    loadChildren: () => import('./radar-state/radar-state.routes'),
  },
  {
    path: 'conversation',
    data: { pageTitle: 'jobSearchRustApp.conversation.home.title' },
    loadChildren: () => import('./conversation/conversation.routes'),
  },
  {
    path: 'cv-resume',
    data: { pageTitle: 'jobSearchRustApp.cvResume.home.title' },
    loadChildren: () => import('./cv-resume/cv-resume.routes'),
  },
  {
    path: 'cv-resume-version',
    data: { pageTitle: 'jobSearchRustApp.cvResumeVersion.home.title' },
    loadChildren: () => import('./cv-resume-version/cv-resume-version.routes'),
  },
  {
    path: 'offer-positioning',
    data: { pageTitle: 'jobSearchRustApp.offerPositioning.home.title' },
    loadChildren: () => import('./offer-positioning/offer-positioning.routes'),
  },
  {
    path: 'offer-tailored-resume',
    data: { pageTitle: 'jobSearchRustApp.offerTailoredResume.home.title' },
    loadChildren: () => import('./offer-tailored-resume/offer-tailored-resume.routes'),
  },
  {
    path: 'user-management',
    data: { pageTitle: 'userManagement.home.title' },
    loadChildren: () => import('./admin/user-management/user-management.routes'),
  },
  /* jhipster-needle-add-entity-route - JHipster will add entity modules routes here */
];

export default routes;
