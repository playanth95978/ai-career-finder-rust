import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import CvResumeVersionResolve from './route/cv-resume-version-routing-resolve.service';

const cvResumeVersionRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/cv-resume-version').then(m => m.CvResumeVersion),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/cv-resume-version-detail').then(m => m.CvResumeVersionDetail),
    resolve: {
      cvResumeVersion: CvResumeVersionResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/cv-resume-version-update').then(m => m.CvResumeVersionUpdate),
    resolve: {
      cvResumeVersion: CvResumeVersionResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/cv-resume-version-update').then(m => m.CvResumeVersionUpdate),
    resolve: {
      cvResumeVersion: CvResumeVersionResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default cvResumeVersionRoute;
