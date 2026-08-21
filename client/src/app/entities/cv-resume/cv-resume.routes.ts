import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import CvResumeResolve from './route/cv-resume-routing-resolve.service';

const cvResumeRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/cv-resume').then(m => m.CvResume),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/cv-resume-detail').then(m => m.CvResumeDetail),
    resolve: {
      cvResume: CvResumeResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/cv-resume-update').then(m => m.CvResumeUpdate),
    resolve: {
      cvResume: CvResumeResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/cv-resume-update').then(m => m.CvResumeUpdate),
    resolve: {
      cvResume: CvResumeResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default cvResumeRoute;
