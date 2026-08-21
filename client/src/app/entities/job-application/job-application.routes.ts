import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import JobApplicationResolve from './route/job-application-routing-resolve.service';

const jobApplicationRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/job-application').then(m => m.JobApplication),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/job-application-detail').then(m => m.JobApplicationDetail),
    resolve: {
      jobApplication: JobApplicationResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/job-application-update').then(m => m.JobApplicationUpdate),
    resolve: {
      jobApplication: JobApplicationResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/job-application-update').then(m => m.JobApplicationUpdate),
    resolve: {
      jobApplication: JobApplicationResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default jobApplicationRoute;
