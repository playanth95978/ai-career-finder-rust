import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import JobOfferResolve from './route/job-offer-routing-resolve.service';

const jobOfferRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/job-offer').then(m => m.JobOffer),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/job-offer-detail').then(m => m.JobOfferDetail),
    resolve: {
      jobOffer: JobOfferResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/job-offer-update').then(m => m.JobOfferUpdate),
    resolve: {
      jobOffer: JobOfferResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/job-offer-update').then(m => m.JobOfferUpdate),
    resolve: {
      jobOffer: JobOfferResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default jobOfferRoute;
